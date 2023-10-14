using System.Collections.Generic;
using System.Linq;
using Streamduck.Devices;
using Streamduck.Plugins.Loaders;
using Streamduck.Plugins.Methods;

namespace Streamduck.Plugins;

public sealed class WrappedPlugin {
	private readonly PluginLoadContext _originatedFrom;

	public Plugin Instance { get; }
	
	public WrappedPlugin(Plugin instance, PluginLoadContext originatedFrom) {
		Instance = instance;
		Name = Instance.Name;
		_originatedFrom = originatedFrom;
		Drivers = Instance.Drivers
			.Select(d => new WrappedDriver(this, d))
			.ToArray();

		var methods = PluginReflector.GetMethods(instance).ToArray();

		Actions = Instance.Actions
			.Concat(PluginReflector.AnalyzeActions(methods, instance))
			.Select(a => new WrappedPluginAction(this, a))
			.ToArray();
		Functions = Instance.Functions
			.Concat(PluginReflector.AnalyzeFunctions(methods, instance))
			.Select(f => new WrappedPluginFunction(this, f))
			.ToArray();
		AsyncActions = Instance.AsyncActions
			.Concat(PluginReflector.AnalyzeAsyncActions(methods, instance))
			.Select(a => new WrappedAsyncPluginAction(this, a))
			.ToArray();
		AsyncFunctions = Instance.AsyncFunctions
			.Concat(PluginReflector.AnalyzeAsyncFunctions(methods, instance))
			.Select(f => new WrappedAsyncPluginFunction(this, f))
			.ToArray();
	}

	public string Name { get; }

	public IEnumerable<WrappedDriver> Drivers { get; }
	public IEnumerable<WrappedPluginAction> Actions { get; }
	public IEnumerable<WrappedPluginFunction> Functions { get; }
	public IEnumerable<WrappedAsyncPluginAction> AsyncActions { get; }
	public IEnumerable<WrappedAsyncPluginFunction> AsyncFunctions { get; }

	public NamespacedName NamespaceName(string name) =>
		new(Name, name);

	public bool BelongsTo(PluginAssembly assembly) => assembly.Context.Equals(_originatedFrom);
}