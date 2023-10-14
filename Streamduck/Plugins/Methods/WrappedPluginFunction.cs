namespace Streamduck.Plugins.Methods; 

public class WrappedPluginFunction {
	public PluginFunction Instance { get; }
	public NamespacedName Name { get; }

	public WrappedPluginFunction(WrappedPlugin plugin, PluginFunction instance) {
		Instance = instance;
		Name = plugin.NamespaceName(instance.Name);
	}
}