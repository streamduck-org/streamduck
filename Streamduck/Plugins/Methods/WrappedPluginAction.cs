namespace Streamduck.Plugins.Methods; 

public class WrappedPluginAction {
	public PluginAction Instance { get; }
	public NamespacedName Name { get; }

	public WrappedPluginAction(WrappedPlugin plugin, PluginAction instance) {
		Instance = instance;
		Name = plugin.NamespaceName(instance.Name);
	}
}