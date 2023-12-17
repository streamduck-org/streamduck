using System.Collections.Generic;
using System.Linq;
using Streamduck.Data;
using Streamduck.Interfaces;
using Streamduck.Plugins.Loaders;
using Streamduck.Rendering;

namespace Streamduck.Plugins;

/**
 * Plugin wrapper that includes namespaced versions of all plugin types
 */
public sealed class WrappedPlugin {
	private readonly PluginLoadContext _originatedFrom;

	public AggregatedPlugin Instance { get; }
	
	public WrappedPlugin(Plugin instance, PluginLoadContext originatedFrom) {
		Instance = new AggregatedPlugin(instance);
		_originatedFrom = originatedFrom;
		
		Name = Instance.Name;
		Drivers = Instance.Drivers
			.Select(Namespace)
			.ToArray();
		Actions = Instance.Actions
			.Select(Namespace)
			.ToArray();
		AsyncActions = Instance.Actions
			.Select(Namespace)
			.ToArray();
		Renderers = Instance.Renderers
			.Select(Namespace)
			.ToArray();
	}

	public string Name { get; }

	public IEnumerable<Namespaced<Driver>> Drivers { get; }
	public IEnumerable<Namespaced<PluginAction>> Actions { get; }
	public IEnumerable<Namespaced<PluginAction>> AsyncActions { get; }
	public IEnumerable<Namespaced<Renderer>> Renderers { get; }

	public Namespaced<T> Namespace<T>(T instance) where T : class, INamed =>
		new(new NamespacedName(Name, instance.Name), instance);

	public bool BelongsTo(PluginAssembly assembly) => assembly.Context.Equals(_originatedFrom);
}