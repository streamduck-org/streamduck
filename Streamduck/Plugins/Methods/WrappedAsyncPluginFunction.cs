namespace Streamduck.Plugins.Methods; 

public class WrappedAsyncPluginFunction {
	public AsyncPluginFunction Instance { get; }
	public NamespacedName Name { get; }

	public WrappedAsyncPluginFunction(WrappedPlugin plugin, AsyncPluginFunction instance) {
		Instance = instance;
		Name = plugin.NamespaceName(instance.Name);
	}
}