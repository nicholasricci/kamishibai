//! Undoable edit commands: the foundation of Kamishibai's AI-first design.
//!
//! Every change to a project (moving an entity, renaming it, spawning a block)
//! goes through an [`EditCommand`]. The editor UI, scripts and AI assistants all
//! use the same commands, so every change can be undone, redone and reviewed,
//! no matter who made it.
//!
//! ```
//! use bevy::prelude::*;
//! use kamishibai::prelude::*;
//!
//! let mut world = World::new();
//! world.init_resource::<EditHistory>();
//! let entity = world.spawn(Transform::default()).id();
//!
//! world.edit(SetTransform::new(entity, Transform::from_xyz(1.0, 0.0, 0.0)));
//! assert_eq!(world.get::<Transform>(entity).unwrap().translation.x, 1.0);
//!
//! world.undo_edit();
//! assert_eq!(world.get::<Transform>(entity).unwrap().translation.x, 0.0);
//! ```

use bevy::prelude::*;

/// A reversible change to the world.
///
/// `apply` is called when the command is executed or redone, `undo` when it is
/// undone. Implementations should store whatever they need to restore the
/// previous state the first time `apply` runs.
pub trait EditCommand: Send + Sync + 'static {
    /// Short, human readable description shown in the editor history
    /// (and to AI assistants), e.g. "Move Player".
    fn label(&self) -> String;

    /// Performs (or re-performs) the change.
    fn apply(&mut self, world: &mut World);

    /// Reverts the change made by the last call to [`EditCommand::apply`].
    fn undo(&mut self, world: &mut World);
}

/// Undo/redo stacks for [`EditCommand`]s.
#[derive(Resource, Default)]
pub struct EditHistory {
    done: Vec<Box<dyn EditCommand>>,
    undone: Vec<Box<dyn EditCommand>>,
}

impl EditHistory {
    /// Labels of the commands that can be undone, oldest first.
    pub fn undo_labels(&self) -> Vec<String> {
        self.done.iter().map(|c| c.label()).collect()
    }

    /// Labels of the commands that can be redone, next one last.
    pub fn redo_labels(&self) -> Vec<String> {
        self.undone.iter().map(|c| c.label()).collect()
    }

    /// Whether there is anything to undo.
    pub fn can_undo(&self) -> bool {
        !self.done.is_empty()
    }

    /// Whether there is anything to redo.
    pub fn can_redo(&self) -> bool {
        !self.undone.is_empty()
    }
}

/// Runs edit commands on a [`World`] while recording them in [`EditHistory`].
///
/// The world must contain an [`EditHistory`] resource
/// (added by [`KamishibaiPlugin`](crate::KamishibaiPlugin)).
pub trait WorldEditExt {
    /// Applies `command` and records it. Clears the redo stack.
    fn edit(&mut self, command: impl EditCommand);
    /// Undoes the most recent command. Returns `false` if there was none.
    fn undo_edit(&mut self) -> bool;
    /// Redoes the most recently undone command. Returns `false` if there was none.
    fn redo_edit(&mut self) -> bool;
}

impl WorldEditExt for World {
    fn edit(&mut self, command: impl EditCommand) {
        let mut command: Box<dyn EditCommand> = Box::new(command);
        command.apply(self);
        let mut history = self.resource_mut::<EditHistory>();
        history.done.push(command);
        history.undone.clear();
    }

    fn undo_edit(&mut self) -> bool {
        let Some(mut command) = self.resource_mut::<EditHistory>().done.pop() else {
            return false;
        };
        command.undo(self);
        self.resource_mut::<EditHistory>().undone.push(command);
        true
    }

    fn redo_edit(&mut self) -> bool {
        let Some(mut command) = self.resource_mut::<EditHistory>().undone.pop() else {
            return false;
        };
        command.apply(self);
        self.resource_mut::<EditHistory>().done.push(command);
        true
    }
}

/// Lets systems queue edits through Bevy's [`Commands`].
pub trait CommandsEditExt {
    /// Queues `command` to be applied and recorded.
    fn edit(&mut self, command: impl EditCommand);
    /// Queues an undo of the most recent edit.
    fn undo_edit(&mut self);
    /// Queues a redo of the most recently undone edit.
    fn redo_edit(&mut self);
}

impl CommandsEditExt for Commands<'_, '_> {
    fn edit(&mut self, command: impl EditCommand) {
        self.queue(move |world: &mut World| world.edit(command));
    }

    fn undo_edit(&mut self) {
        self.queue(|world: &mut World| {
            world.undo_edit();
        });
    }

    fn redo_edit(&mut self) {
        self.queue(|world: &mut World| {
            world.redo_edit();
        });
    }
}

/// Sets an entity's [`Transform`].
pub struct SetTransform {
    entity: Entity,
    new: Transform,
    old: Option<Transform>,
}

impl SetTransform {
    /// Creates a command that moves `entity` to `transform`.
    pub fn new(entity: Entity, transform: Transform) -> Self {
        Self {
            entity,
            new: transform,
            old: None,
        }
    }
}

impl EditCommand for SetTransform {
    fn label(&self) -> String {
        format!("Set transform of {}", self.entity)
    }

    fn apply(&mut self, world: &mut World) {
        if let Some(mut transform) = world.get_mut::<Transform>(self.entity) {
            self.old = Some(*transform);
            *transform = self.new;
        }
    }

    fn undo(&mut self, world: &mut World) {
        if let (Some(old), Some(mut transform)) =
            (self.old, world.get_mut::<Transform>(self.entity))
        {
            *transform = old;
        }
    }
}

/// Sets (or removes, with `None`) an entity's [`Name`].
pub struct SetName {
    entity: Entity,
    new: Option<Name>,
    old: Option<Name>,
}

impl SetName {
    /// Creates a command that renames `entity`.
    pub fn new(entity: Entity, name: Option<impl Into<Name>>) -> Self {
        Self {
            entity,
            new: name.map(Into::into),
            old: None,
        }
    }
}

impl EditCommand for SetName {
    fn label(&self) -> String {
        match &self.new {
            Some(name) => format!("Rename {} to \"{name}\"", self.entity),
            None => format!("Remove name of {}", self.entity),
        }
    }

    fn apply(&mut self, world: &mut World) {
        let Ok(mut entity) = world.get_entity_mut(self.entity) else {
            return;
        };
        self.old = entity.take::<Name>();
        if let Some(name) = self.new.clone() {
            entity.insert(name);
        }
    }

    fn undo(&mut self, world: &mut World) {
        let Ok(mut entity) = world.get_entity_mut(self.entity) else {
            return;
        };
        entity.remove::<Name>();
        if let Some(name) = self.old.clone() {
            entity.insert(name);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn world_with_history() -> World {
        let mut world = World::new();
        world.init_resource::<EditHistory>();
        world
    }

    fn x(world: &World, entity: Entity) -> f32 {
        world.get::<Transform>(entity).unwrap().translation.x
    }

    #[test]
    fn undo_and_redo_transform() {
        let mut world = world_with_history();
        let e = world.spawn(Transform::default()).id();

        world.edit(SetTransform::new(e, Transform::from_xyz(1.0, 0.0, 0.0)));
        world.edit(SetTransform::new(e, Transform::from_xyz(2.0, 0.0, 0.0)));
        assert_eq!(x(&world, e), 2.0);

        assert!(world.undo_edit());
        assert_eq!(x(&world, e), 1.0);
        assert!(world.undo_edit());
        assert_eq!(x(&world, e), 0.0);
        assert!(!world.undo_edit());

        assert!(world.redo_edit());
        assert_eq!(x(&world, e), 1.0);
    }

    #[test]
    fn new_edit_clears_redo() {
        let mut world = world_with_history();
        let e = world.spawn(Transform::default()).id();

        world.edit(SetTransform::new(e, Transform::from_xyz(1.0, 0.0, 0.0)));
        world.undo_edit();
        assert!(world.resource::<EditHistory>().can_redo());

        world.edit(SetTransform::new(e, Transform::from_xyz(5.0, 0.0, 0.0)));
        assert!(!world.resource::<EditHistory>().can_redo());
        assert!(!world.redo_edit());
    }

    #[test]
    fn rename_and_undo() {
        let mut world = world_with_history();
        let e = world.spawn(Name::new("Old")).id();

        world.edit(SetName::new(e, Some("Hero")));
        assert_eq!(world.get::<Name>(e).unwrap().as_str(), "Hero");
        assert_eq!(
            world.resource::<EditHistory>().undo_labels(),
            vec![format!("Rename {e} to \"Hero\"")]
        );

        world.undo_edit();
        assert_eq!(world.get::<Name>(e).unwrap().as_str(), "Old");
    }

    #[test]
    fn edits_on_missing_entities_are_harmless() {
        let mut world = world_with_history();
        let e = world.spawn(Transform::default()).id();
        world.despawn(e);

        world.edit(SetTransform::new(e, Transform::from_xyz(1.0, 0.0, 0.0)));
        world.edit(SetName::new(e, Some("Ghost")));
        assert!(world.undo_edit());
        assert!(world.undo_edit());
    }

    #[test]
    fn commands_queue_edits() {
        let mut world = world_with_history();
        let e = world.spawn(Transform::default()).id();

        let mut queue = bevy::ecs::world::CommandQueue::default();
        let mut commands = Commands::new(&mut queue, &world);
        commands.edit(SetTransform::new(e, Transform::from_xyz(3.0, 0.0, 0.0)));
        commands.undo_edit();
        commands.redo_edit();
        queue.apply(&mut world);

        assert_eq!(x(&world, e), 3.0);
    }
}
