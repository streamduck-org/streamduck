using Streamduck.Plugins;

namespace Streamduck.Data; 

/**
 * Type that belongs to a plugin
 */
public class Namespaced<T> where T : class {
	public NamespacedName NamespacedName { get; } 
	public T Instance { get; }

	public string Name => NamespacedName.Name;
	public string PluginName => NamespacedName.PluginName;

	public Namespaced(NamespacedName namespacedName, T instance) {
		NamespacedName = namespacedName;
		Instance = instance;
	}
}