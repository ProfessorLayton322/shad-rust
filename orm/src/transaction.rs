use crate::{
    data::ObjectId,
    error::*,
    object::{fetch_id, fetch_schema, Object, Schema, Store},
    storage::StorageTransaction,
};

use std::{
    any::Any,
    cell::{Ref, RefCell, RefMut},
    collections::HashMap,
    marker::PhantomData,
    ops::Deref,
    ops::DerefMut,
    rc::Rc,
};

////////////////////////////////////////////////////////////////////////////////

type Key = (&'static str, ObjectId);
type ObjectWrapper = (ObjectState, Box<dyn Store>);

pub struct Transaction<'a> {
    inner: Box<dyn StorageTransaction + 'a>,
    content: RefCell<HashMap<Key, Rc<RefCell<ObjectWrapper>>>>,
}

impl<'a> Transaction<'a> {
    pub(crate) fn new(inner: Box<dyn StorageTransaction + 'a>) -> Self {
        Self {
            inner,
            content: RefCell::new(HashMap::new()),
        }
    }

    pub fn create<T: Object>(&self, obj: T) -> Result<Tx<'_, T>> {
        let schema: &Schema = &T::SCHEMA;
        fetch_schema(schema);
        if !self.inner.table_exists(schema.table_name)? {
            self.inner.create_table(schema)?;
        }
        let id = self.inner.insert_row(schema, &obj.to_row())?;
        let rc = self
            .content
            .borrow_mut()
            .entry((schema.table_name, id))
            .or_insert_with(|| Rc::new(RefCell::new((ObjectState::Clean, Box::new(obj)))))
            .clone();
        Ok(Tx {
            inner: rc,
            id,
            lifetime: PhantomData::<&'_ Transaction>,
            holder: PhantomData::<&'_ T>,
        })
    }

    pub fn get<T: Object>(&self, id: ObjectId) -> Result<Tx<'_, T>> {
        let schema: &Schema = &T::SCHEMA;
        if !self.inner.table_exists(schema.table_name)? {
            return Err(Error::NotFound(Box::new(NotFoundError {
                object_id: id,
                type_name: schema.struct_name,
            })));
        }
        fetch_schema(schema);
        fetch_id(id);
        if let Some(rc) = self.content.borrow().get(&(schema.table_name, id)) {
            if let ObjectState::Removed = rc.borrow().0 {
                return Err(Error::NotFound(Box::new(NotFoundError {
                    object_id: id,
                    type_name: schema.struct_name,
                })));
            }
            return Ok(Tx {
                inner: rc.clone(),
                id,
                lifetime: PhantomData::<&'_ Transaction>,
                holder: PhantomData::<&'_ T>,
            });
        }
        let row = self.inner.select_row(id, schema)?;
        let obj = T::from_row(&row);
        let rc: Rc<RefCell<(ObjectState, Box<dyn Store>)>> =
            Rc::new(RefCell::new((ObjectState::Clean, Box::new(obj))));
        self.content
            .borrow_mut()
            .insert((schema.table_name, id), rc.clone());
        Ok(Tx {
            inner: rc,
            id,
            lifetime: PhantomData::<&'_ Transaction>,
            holder: PhantomData::<&'_ T>,
        })
    }

    pub fn commit(self) -> Result<()> {
        for ((_, id), rc) in self.content.borrow().iter() {
            let schema: &'static Schema = rc.borrow().1.get_schema();
            fetch_id(*id);
            match rc.borrow().0 {
                ObjectState::Clean => continue,
                ObjectState::Removed => self.inner.delete_row(*id, schema)?,
                ObjectState::Modified => {
                    let store = &rc.borrow().1;
                    let row = store.get_row();
                    self.inner.update_row(*id, schema, &row)?;
                }
            }
        }
        self.inner.commit()?;
        Ok(())
    }

    pub fn rollback(self) -> Result<()> {
        self.inner.rollback()?;
        Ok(())
    }
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ObjectState {
    Clean,
    Modified,
    Removed,
}

#[derive(Clone)]
pub struct Tx<'a, T> {
    inner: Rc<RefCell<(ObjectState, Box<dyn Store>)>>,
    id: ObjectId,
    lifetime: PhantomData<&'a Transaction<'a>>,
    holder: PhantomData<&'a T>,
}

impl<'a, T: Any> Tx<'a, T> {
    pub fn id(&self) -> ObjectId {
        self.id
    }

    pub fn state(&self) -> ObjectState {
        self.inner.borrow().0
    }

    pub fn borrow(&self) -> RefWrapper<'_, T> {
        if let ObjectState::Removed = self.inner.borrow().0 {
            panic!("cannot borrow a removed object");
        }
        RefWrapper {
            inner: Ref::map(self.inner.borrow(), |val| &val.1),
            lifetime: PhantomData::<&'_ Tx<T>>,
        }
    }

    pub fn borrow_mut(&self) -> RefWrapperMut<'_, T> {
        if let ObjectState::Removed = self.inner.borrow().0 {
            panic!();
        }
        self.inner.borrow_mut().0 = ObjectState::Modified;
        RefWrapperMut {
            inner: RefMut::map(self.inner.borrow_mut(), |val| &mut val.1),
            lifetime: PhantomData::<&'_ Tx<'a, T>>,
        }
    }

    pub fn delete(self) {
        let Ok(mut mutref) = self.inner.try_borrow_mut() else {
            panic!("cannot delete a borrowed object");
        };
        mutref.0 = ObjectState::Removed;
    }
}

pub struct RefWrapper<'a, T> {
    inner: Ref<'a, Box<dyn Store>>,
    lifetime: PhantomData<&'a Tx<'a, T>>,
}

impl<'a, T: 'static> Deref for RefWrapper<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.inner.deref().as_any().downcast_ref::<T>().unwrap()
    }
}

pub struct RefWrapperMut<'a, T> {
    inner: RefMut<'a, Box<dyn Store>>,
    lifetime: PhantomData<&'a Tx<'a, T>>,
}

impl<'a, T: 'static> Deref for RefWrapperMut<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.inner.deref().as_any().downcast_ref::<T>().unwrap()
    }
}

impl<'a, T: 'static> DerefMut for RefWrapperMut<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.inner
            .deref_mut()
            .as_any_mut()
            .downcast_mut::<T>()
            .unwrap()
    }
}
