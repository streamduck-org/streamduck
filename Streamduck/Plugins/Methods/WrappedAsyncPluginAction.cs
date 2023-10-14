namespace Streamduck.Plugins.Methods; 

public class WrappedAsyncPluginAction {
	public AsyncPluginAction Instance { get; }
	public NamespacedName Name { get; }

	public WrappedAsyncPluginAction(WrappedPlugin plugin, AsyncPluginAction instance) {
		Instance = instance;
		Name = plugin.NamespaceName(instance.Name);
	}
}