using System.Collections.Generic;
using System.Linq;
using System.Threading.Tasks;
using Streamduck.Rendering;

namespace Streamduck.Plugins; 

/**
 * Plugin wrapper that runs reflection on inner instance
 */
public class AggregatedPlugin : Plugin {
	public AggregatedPlugin(Plugin instance) {
		Instance = instance;
		Name = Instance.Name;
		Drivers = Instance.Drivers.ToArray();
		Renderers = Instance.Renderers.ToArray();

		var methods = PluginReflector.GetMethods(instance).ToArray();
		
		Actions = Instance.Actions
			.Concat(PluginReflector.AnalyzeActions(methods, instance))
			.ToArray();
	}
	public Plugin Instance { get; }
	public override string Name { get; }
	public override IEnumerable<Driver> Drivers { get; }
	public override IEnumerable<PluginAction> Actions { get; }
	public override IEnumerable<Renderer> Renderers { get; }

	public override Task OnPluginsLoaded(IPluginQuery pluginQuery) => Instance.OnPluginsLoaded(pluginQuery);
}