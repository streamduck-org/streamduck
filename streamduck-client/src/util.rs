use std::collections::HashMap;
use streamduck_core::modules::components::ComponentDefinition;

/// Transforms module-component map into component map, if you don't care about module names for em
pub fn module_component_map_to_component_map(component_map: HashMap<String, HashMap<String, ComponentDefinition>>) -> HashMap<String, ComponentDefinition> {
    let mut map = HashMap::new();

    for (_, components) in component_map {
        map.extend(components)
    }

    map
}