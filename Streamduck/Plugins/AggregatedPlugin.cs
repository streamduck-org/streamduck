using System.Collections.Generic;
using System.Linq;
using System.Threading.Tasks;
using Streamduck.Scripting;

namespace Streamduck.Plugins; 

/**
 * Plugin wrapper that runs reflection on inner instance
 */
public class AggregatedPlugin : Plugin {
	public AggregatedPlugin(Plugin instance) {
		Instance = instance;
		Name = Instance.Name;
		Drivers = Instance.Drivers.ToArray();
		ScriptingSystems = Instance.ScriptingSystems.ToArray();

		var methods = PluginReflector.GetMethods(instance).ToArray();

		Actions = Instance.Actions
			.Concat(PluginReflector.AnalyzeActions(methods, instance))
			.ToArray();
		Functions = Instance.Functions
			.Concat(PluginReflector.AnalyzeFunctions(methods, instance))
			.ToArray();
		AsyncActions = Instance.AsyncActions
			.Concat(PluginReflector.AnalyzeAsyncActions(methods, instance))
			.ToArray();
		AsyncFunctions = Instance.AsyncFunctions
			.Concat(PluginReflector.AnalyzeAsyncFunctions(methods, instance))
			.ToArray();
	}
	public Plugin Instance { get; }
	public override string Name { get; }
	public override IEnumerable<Driver> Drivers { get; }
	public override IEnumerable<PluginAction> Actions { get; }
	public override IEnumerable<PluginFunction> Functions { get; }
	public override IEnumerable<AsyncPluginAction> AsyncActions { get; }
	public override IEnumerable<AsyncPluginFunction> AsyncFunctions { get; }
	public override IEnumerable<ScriptingSystem> ScriptingSystems { get; }

	public override Task OnPluginsLoaded(IPluginQuery pluginQuery) => Instance.OnPluginsLoaded(pluginQuery);
}