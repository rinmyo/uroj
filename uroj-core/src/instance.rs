use super::map::Map;

struct Instance {
    id: String,
    map: Map,
}

impl Instance {
    fn new(id_generator: fn() -> String, map: Map) -> Self {
        let id = id_generator();
        println!("Instance Created!: {:?}", id);
        Instance { id, map }
    }
}

impl Drop for Instance {
    fn drop(&mut self) {
        println!("Instance Dropped!: {:?}", self.id)
    }
}

pub struct InstancePool {
    data: Vec<Instance>,
}

impl InstancePool {
    fn new() -> Self {
        InstancePool { data: Vec::new() }
    }

    fn remove(&mut self) {
        self.data.remove(index: usize)
    }
}

impl Drop for InstancePool {
    fn drop(&mut self) {
        println!("{:?}", "InstancePool Dropped!")
    }
}
