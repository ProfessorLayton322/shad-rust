use std::convert::From;
use std::ptr::null;
use thiserror::Error;

use std::{collections::HashMap, iter::Iterator, marker::PhantomData, ops::Deref};

////////////////////////////////////////////////////////////////////////////////

pub type ObjectId = i64;

////////////////////////////////////////////////////////////////////////////////

#[derive(PartialEq, Eq, Clone, Copy, Default, Debug)]
pub struct ResourceTotals {
    pub cpu: u64,
    pub memory: u64,
    pub disk_capacity: u64,
}

impl std::ops::AddAssign for ResourceTotals {
    fn add_assign(&mut self, other: Self) {
        self.cpu += other.cpu;
        self.memory += other.memory;
        self.disk_capacity += other.disk_capacity;
    }
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Default)]
pub struct Dump {
    pub node_segments: Vec<NodeSegmentRecord>,
    pub nodes: Vec<NodeRecord>,
    pub pod_sets: Vec<PodSetRecord>,
    pub pods: Vec<PodRecord>,
}

pub struct NodeSegmentRecord {
    pub id: ObjectId,
}

pub struct NodeRecord {
    pub id: ObjectId,
    pub node_segment_id: ObjectId,
    pub resource_totals: ResourceTotals,
}

pub struct PodSetRecord {
    pub id: ObjectId,
    pub node_segment_id: ObjectId,
}

pub struct PodRecord {
    pub id: ObjectId,
    pub pod_set_id: ObjectId,
    pub node_id: Option<ObjectId>,
    pub resource_requests: ResourceTotals,
}

////////////////////////////////////////////////////////////////////////////////

pub struct Sn<'a, T> {
    ptr: *const T,
    lifetime: PhantomData<&'a Snapshot>,
}

impl<'a, T> Deref for Sn<'a, T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { &*self.ptr }
    }
}

////////////////////////////////////////////////////////////////////////////////

pub struct NodeSegment {
    pub id: ObjectId,
    pub resource_usage: ResourceTotals,
    pub resource_requests: ResourceTotals,
    pub resource_totals: ResourceTotals,
    nodes: Vec<*const Node>,
    pod_sets: Vec<*const PodSet>,
}

impl<'a> Sn<'a, NodeSegment> {
    pub fn nodes(&self) -> impl Iterator<Item = Sn<'a, Node>> + '_ {
        let node_segment = self.deref();
        node_segment.nodes.iter().map(|ptr| Sn::<'a, Node> {
            ptr: *ptr,
            lifetime: self.lifetime,
        })
    }

    pub fn pod_sets(&self) -> impl Iterator<Item = Sn<'a, PodSet>> + '_ {
        let node_segment = self.deref();
        node_segment.pod_sets.iter().map(|ptr| Sn::<'a, PodSet> {
            ptr: *ptr,
            lifetime: self.lifetime,
        })
    }
}

impl From<&NodeSegmentRecord> for NodeSegment {
    fn from(record: &NodeSegmentRecord) -> Self {
        Self {
            id: record.id,
            resource_usage: ResourceTotals::default(),
            resource_requests: ResourceTotals::default(),
            resource_totals: ResourceTotals::default(),
            nodes: Vec::new(),
            pod_sets: Vec::new(),
        }
    }
}

impl NodeSegment {
    pub fn push_node(&mut self, node: &mut Node) {
        self.nodes.push(node);
        node.node_segment = self;
        self.resource_usage += node.resource_usage;
        self.resource_totals += node.resource_totals;
    }

    pub fn push_pod_set(&mut self, pod_set: &mut PodSet) {
        self.pod_sets.push(pod_set);
        pod_set.node_segment = self;
        self.resource_requests += pod_set.resource_requests;
    }
}

////////////////////////////////////////////////////////////////////////////////

pub struct Node {
    pub id: ObjectId,
    pub resource_usage: ResourceTotals,
    pub resource_totals: ResourceTotals,
    node_segment: *const NodeSegment,
    pods: Vec<*const Pod>,
}

impl<'a> Sn<'a, Node> {
    pub fn pods(&self) -> impl Iterator<Item = Sn<'a, Pod>> + '_ {
        let node = self.deref();
        node.pods.iter().map(|ptr| Sn::<'a, Pod> {
            ptr: *ptr,
            lifetime: self.lifetime,
        })
    }

    pub fn node_segment(&self) -> Sn<'a, NodeSegment> {
        let node = self.deref();
        Sn {
            ptr: node.node_segment,
            lifetime: self.lifetime,
        }
    }
}

impl From<&NodeRecord> for Node {
    fn from(record: &NodeRecord) -> Self {
        Self {
            id: record.id,
            resource_usage: ResourceTotals::default(),
            resource_totals: record.resource_totals,
            node_segment: null(),
            pods: Vec::new(),
        }
    }
}

impl Node {
    pub fn push_pod(&mut self, pod: &mut Pod) {
        self.pods.push(pod);
        pod.node = Some(self as *const Node);
        self.resource_usage += pod.resource_requests;
    }
}

////////////////////////////////////////////////////////////////////////////////

pub struct PodSet {
    pub id: ObjectId,
    pub resource_requests: ResourceTotals,
    node_segment: *const NodeSegment,
    pods: Vec<*const Pod>,
}

impl<'a> Sn<'a, PodSet> {
    pub fn pods(&self) -> impl Iterator<Item = Sn<'a, Pod>> + '_ {
        let pod_set = self.deref();
        pod_set.pods.iter().map(|ptr| Sn::<'a, Pod> {
            ptr: *ptr,
            lifetime: self.lifetime,
        })
    }

    pub fn node_segment(&self) -> Sn<'a, NodeSegment> {
        let pod_set = self.deref();
        Sn {
            ptr: pod_set.node_segment,
            lifetime: self.lifetime,
        }
    }
}

impl From<&PodSetRecord> for PodSet {
    fn from(record: &PodSetRecord) -> Self {
        Self {
            id: record.id,
            resource_requests: ResourceTotals::default(),
            node_segment: null(),
            pods: Vec::new(),
        }
    }
}

impl PodSet {
    pub fn push_pod(&mut self, pod: &mut Pod) {
        self.pods.push(pod);
        self.resource_requests += pod.resource_requests;
        pod.pod_set = self;
    }
}

////////////////////////////////////////////////////////////////////////////////

pub struct Pod {
    pub id: ObjectId,
    pub resource_requests: ResourceTotals,
    pod_set: *const PodSet,
    node: Option<*const Node>,
}

impl<'a> Sn<'a, Pod> {
    pub fn pod_set(&self) -> Sn<'a, PodSet> {
        let pod = self.deref();
        Sn {
            ptr: pod.pod_set,
            lifetime: self.lifetime,
        }
    }

    pub fn node(&self) -> Option<Sn<'a, Node>> {
        let pod = self.deref();
        pod.node.map(|node| Sn {
            ptr: node,
            lifetime: self.lifetime,
        })
    }
}

impl From<&PodRecord> for Pod {
    fn from(record: &PodRecord) -> Self {
        Self {
            id: record.id,
            resource_requests: record.resource_requests,
            pod_set: null(),
            node: None,
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Default)]
pub struct Snapshot {
    node_segments: HashMap<ObjectId, NodeSegment>,
    pod_sets: HashMap<ObjectId, PodSet>,
    pods: HashMap<ObjectId, Pod>,
    nodes: HashMap<ObjectId, Node>,
}

impl Snapshot {
    pub fn new(dump: &Dump) -> Result<Self> {
        let mut node_segments: HashMap<ObjectId, NodeSegment> = HashMap::new();
        let mut pod_sets: HashMap<ObjectId, PodSet> = HashMap::new();
        let mut pods: HashMap<ObjectId, Pod> = HashMap::new();
        let mut nodes: HashMap<ObjectId, Node> = HashMap::new();

        for resource in dump.node_segments.iter() {
            let item: NodeSegment = resource.into();
            let id = item.id;
            if node_segments.insert(id, item).is_some() {
                return Err(Error::DuplicateObject {
                    ty: ObjectType::NodeSegment,
                    id,
                });
            }
        }

        for resource in dump.nodes.iter() {
            let item: Node = resource.into();
            let id = item.id;
            if nodes.insert(id, item).is_some() {
                return Err(Error::DuplicateObject {
                    ty: ObjectType::Node,
                    id,
                });
            }
        }

        for resource in dump.pods.iter() {
            let item: Pod = resource.into();
            let id = item.id;
            if pods.insert(id, item).is_some() {
                return Err(Error::DuplicateObject {
                    ty: ObjectType::Pod,
                    id,
                });
            }
        }

        for resource in dump.pod_sets.iter() {
            let item: PodSet = resource.into();
            let id = item.id;
            if pod_sets.insert(id, item).is_some() {
                return Err(Error::DuplicateObject {
                    ty: ObjectType::PodSet,
                    id,
                });
            }
        }

        for record in dump.pods.iter() {
            let pod = pods.get_mut(&record.id).unwrap();
            let Some(pod_set) = pod_sets.get_mut(&record.pod_set_id) else {
                return Err(Error::MissingObject { ty: ObjectType::PodSet, id: record.pod_set_id });
            };
            pod_set.push_pod(pod);
            if let Some(node_id) = record.node_id {
                let Some(node) = nodes.get_mut(&node_id) else {
                    return Err(Error::MissingObject { ty: ObjectType::Node, id: node_id });
                };
                node.push_pod(pod);
            }
        }

        for record in dump.pod_sets.iter() {
            let pod_set = pod_sets.get_mut(&record.id).unwrap();
            let Some(node_segment) = node_segments.get_mut(&record.node_segment_id) else {
                return Err(Error::MissingObject { ty: ObjectType::NodeSegment, id: record.node_segment_id });
            };
            node_segment.push_pod_set(pod_set);
        }

        for record in dump.nodes.iter() {
            let node = nodes.get_mut(&record.id).unwrap();
            let Some(node_segment) = node_segments.get_mut(&record.node_segment_id) else {
                return Err(Error::MissingObject { ty: ObjectType::NodeSegment, id: record.node_segment_id });
            };
            node_segment.push_node(node);
        }

        Ok(Self {
            node_segments,
            pod_sets,
            pods,
            nodes,
        })
    }

    pub fn get_node(&self, id: &ObjectId) -> Result<Sn<'_, Node>> {
        let Some(node) = self.nodes.get(id) else {
            return Err(Error::MissingObject { ty : ObjectType::Node, id: *id });
        };
        Ok(Sn {
            ptr: node,
            lifetime: PhantomData::<&'_ Snapshot>,
        })
    }

    pub fn get_node_segment(&self, id: &ObjectId) -> Result<Sn<'_, NodeSegment>> {
        let Some(node_segment) = self.node_segments.get(id) else {
            return Err(Error::MissingObject { ty : ObjectType::NodeSegment, id: *id });
        };
        Ok(Sn {
            ptr: node_segment,
            lifetime: PhantomData::<&'_ Snapshot>,
        })
    }

    pub fn get_pod(&self, id: &ObjectId) -> Result<Sn<'_, Pod>> {
        let Some(pod) = self.pods.get(id) else {
            return Err(Error::MissingObject { ty : ObjectType::Pod, id: *id });
        };
        Ok(Sn {
            ptr: pod,
            lifetime: PhantomData::<&'_ Snapshot>,
        })
    }

    pub fn get_pod_set(&self, id: &ObjectId) -> Result<Sn<'_, PodSet>> {
        let Some(pod_set) = self.pod_sets.get(id) else {
            return Err(Error::MissingObject { ty : ObjectType::PodSet, id: *id });
        };
        Ok(Sn {
            ptr: pod_set,
            lifetime: PhantomData::<&'_ Snapshot>,
        })
    }

    pub fn nodes(&self) -> Vec<Sn<'_, Node>> {
        self.nodes
            .iter()
            .map(|(_key, item)| Sn {
                ptr: item,
                lifetime: PhantomData::<&'_ Snapshot>,
            })
            .collect()
    }

    pub fn node_segments(&self) -> Vec<Sn<'_, NodeSegment>> {
        self.node_segments
            .iter()
            .map(|(_key, item)| Sn {
                ptr: item,
                lifetime: PhantomData::<&'_ Snapshot>,
            })
            .collect()
    }

    pub fn pods(&self) -> Vec<Sn<'_, Pod>> {
        self.pods
            .iter()
            .map(|(_key, item)| Sn {
                ptr: item,
                lifetime: PhantomData::<&'_ Snapshot>,
            })
            .collect()
    }

    pub fn pod_sets(&self) -> Vec<Sn<'_, PodSet>> {
        self.pod_sets
            .iter()
            .map(|(_key, item)| Sn {
                ptr: item,
                lifetime: PhantomData::<&'_ Snapshot>,
            })
            .collect()
    }
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, PartialEq, Eq)]
pub enum ObjectType {
    NodeSegment,
    Node,
    PodSet,
    Pod,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum Error {
    #[error("snapshot references a non-existent object (type: {ty:?}, id: {id})")]
    MissingObject { ty: ObjectType, id: ObjectId },
    #[error("found duplicate object in snapshot (type: {ty:?}, id: {id})")]
    DuplicateObject { ty: ObjectType, id: ObjectId },
}

pub type Result<T> = std::result::Result<T, Error>;
