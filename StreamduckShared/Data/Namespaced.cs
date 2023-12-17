using Streamduck.Plugins;

namespace Streamduck.Data; 

/**
 * Type that belongs to a plugin
 */
public class Namespaced<T>(NamespacedName namespacedName, T instance)
	where T : class {
	public NamespacedName NamespacedName { get; } = namespacedName;
	public T Instance { get; } = instance;

	public string Name => NamespacedName.Name;
	public string PluginName => NamespacedName.PluginName;
}