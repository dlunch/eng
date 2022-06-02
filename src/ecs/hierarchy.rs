use alloc::{vec, vec::Vec};

use crate::ecs::{Component, Entity, World};

pub struct Children {
    pub children: Vec<Entity>,
}
impl Component for Children {}

pub struct Parent {
    pub parent: Entity,
}
impl Component for Parent {}

pub trait HierarchyExt {
    fn add_child(&mut self, entity: Entity, child: Entity);
    fn remove_child(&mut self, entity: Entity, child: Entity) -> bool;
    fn children(&self, entity: Entity) -> &Vec<Entity>;
    fn parent(&self, entity: Entity) -> Option<Entity>;
}

lazy_static::lazy_static! {
    pub static ref EMPTY_VEC: Vec<Entity> = Vec::new();
}

impl HierarchyExt for World {
    fn add_child(&mut self, entity: Entity, child: Entity) {
        let children_component = self.component_mut::<Children>(entity);
        if let Some(x) = children_component {
            x.children.push(child);
        } else {
            self.add_component(entity, Children { children: vec![child] });
        }
    }

    fn remove_child(&mut self, entity: Entity, child: Entity) -> bool {
        let children_component = self.component_mut::<Children>(entity);
        if let Some(x) = children_component {
            if let Some(index) = x.children.iter().position(|&x| x == child) {
                x.children.remove(index);

                true
            } else {
                false
            }
        } else {
            false
        }
    }

    fn children(&self, entity: Entity) -> &Vec<Entity> {
        let children = self.component::<Children>(entity);

        match children {
            Some(children) => &children.children,
            None => &EMPTY_VEC,
        }
    }

    fn parent(&self, entity: Entity) -> Option<Entity> {
        let parent = self.component::<Parent>(entity);

        Some(parent?.parent)
    }
}

#[cfg(test)]
mod test {
    use super::{HierarchyExt, World};

    #[test]
    fn test_add_children() {
        let mut world = World::new();

        let entity = world.spawn().entity();
        let child = world.spawn().entity();

        world.add_child(entity, child);

        assert_eq!(world.children(entity).len(), 1);
        assert_eq!(world.children(entity)[0].id, child.id);
    }

    #[test]
    fn test_remove_children() {
        let mut world = World::new();

        let entity = world.spawn().entity();
        let child = world.spawn().entity();

        assert!(!world.remove_child(entity, child));

        world.add_child(entity, child);
        assert!(world.remove_child(entity, child));

        assert_eq!(world.children(entity).len(), 0);
    }
}
