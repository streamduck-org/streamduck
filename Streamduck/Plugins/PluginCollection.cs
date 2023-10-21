using System;
using System.Collections.Concurrent;
using System.Collections.Generic;
using System.Linq;
using System.Threading.Tasks;
using Streamduck.Data;
using Streamduck.Plugins.Extensions;
using Streamduck.Plugins.Loaders;
using Streamduck.Scripting;
using Streamduck.Utils;

namespace Streamduck.Plugins; 

public class PluginCollection : IPluginQuery {
	public readonly List<PluginAssembly> Plugins;
	
	private readonly ConcurrentDictionary<string, WeakReference<WrappedPlugin>> _pluginMap;
	private readonly ConcurrentDictionary<NamespacedName, WeakReference<Namespaced<Driver>>> _driverMap;
	private readonly ConcurrentDictionary<NamespacedName, WeakReference<Namespaced<PluginAction>>> _actionMap;
	private readonly ConcurrentDictionary<NamespacedName, WeakReference<Namespaced<PluginFunction>>> _functionMap;
	private readonly ConcurrentDictionary<NamespacedName, WeakReference<Namespaced<AsyncPluginAction>>> _asyncActionMap;
	private readonly ConcurrentDictionary<NamespacedName, WeakReference<Namespaced<AsyncPluginFunction>>> _asyncFunctionMap;
	private readonly ConcurrentDictionary<NamespacedName, WeakReference<Namespaced<ScriptingSystem>>> _scriptingSystemMap;

	public PluginCollection(IEnumerable<PluginAssembly> plugins) {
		Plugins = plugins.ToList();
		_pluginMap = new ConcurrentDictionary<string, WeakReference<WrappedPlugin>>(
			Plugins
				.SelectMany(a => a.Plugins)
				.ToDictionary(p => p.Name, p => new WeakReference<WrappedPlugin>(p))
		);
		_driverMap = BuildMap(p => p.Drivers);
		_actionMap = BuildMap(p => p.Actions);
		_functionMap = BuildMap(p => p.Functions);
		_asyncActionMap = BuildMap(p => p.AsyncActions);
		_asyncFunctionMap = BuildMap(p => p.AsyncFunctions);
		_scriptingSystemMap = BuildMap(p => p.ScriptingSystems);
	}

	private static IEnumerable<PluginAssembly> PluginsToAssembly(IEnumerable<Plugin> plugins) {
		var context = new PluginLoadContext("");
		var pluginAssembly = new PluginAssembly(context, plugins.Select(p => new WrappedPlugin(p, context)).ToArray());
		return new[] { pluginAssembly };
	}
	
	public PluginCollection(IEnumerable<Plugin> plugins) : this(PluginsToAssembly(plugins)) {}

	private ConcurrentDictionary<NamespacedName, WeakReference<Namespaced<T>>> BuildMap<T>(
		Func<WrappedPlugin, IEnumerable<Namespaced<T>>> accessor) where T : class =>
		new(
			Plugins
				.SelectMany(a => a.Plugins)
				.SelectMany(accessor)
				.ToDictionary(x => x.NamespacedName, x => new WeakReference<Namespaced<T>>(x))
		);

	private static void AddNamespacedToDict<T>(ConcurrentDictionary<NamespacedName, WeakReference<Namespaced<T>>> dict, 
		IEnumerable<Namespaced<T>> enumerable) where T : class {
		foreach (var x in enumerable) {
			dict.TryAdd(x.NamespacedName, new WeakReference<Namespaced<T>>(x));
		}
	}

	public async Task AddPlugin(PluginAssembly assembly) {
		Plugins.Add(assembly);
		
		foreach (var plugin in assembly.Plugins) {
			AddNamespacedToDict(_driverMap, plugin.Drivers);
			AddNamespacedToDict(_actionMap, plugin.Actions);
			AddNamespacedToDict(_functionMap, plugin.Functions);
			AddNamespacedToDict(_asyncActionMap, plugin.AsyncActions);
			AddNamespacedToDict(_asyncFunctionMap, plugin.AsyncFunctions);
		}

		await this.InvokeNewPluginsLoaded(assembly.Plugins.Select(w => (Plugin)w.Instance).ToArray());
	}
	
	private static IEnumerable<T> GetAll<T>(
		ConcurrentDictionary<NamespacedName, WeakReference<T>> dict) where T : class =>
		from weak in dict
		let strong = weak.Value.WeakToNullable()
		where strong != null
		select strong;
	
	private static IEnumerable<T> GetByPlugin<T>(
		ConcurrentDictionary<NamespacedName, WeakReference<T>> dict,
		string pluginName) where T : class =>
		from weak in dict
		where weak.Key.PluginName == pluginName
		let strong = weak.Value.WeakToNullable()
		where strong != null
		select strong;

	private static T? GetSpecific<T>(
		ConcurrentDictionary<NamespacedName, WeakReference<T>> dict,
		NamespacedName name) where T : class =>
		dict.GetValueOrDefault(name)?.WeakToNullable();

	public IEnumerable<Plugin> AllPlugins() =>
		from weakPlugin in _pluginMap
		let plugin = weakPlugin.Value.WeakToNullable()
		where plugin != null
		select plugin.Instance;

	public Plugin? SpecificPlugin(string name) =>
		_pluginMap.GetValueOrDefault(name)?.WeakToNullable()?.Instance;

	public IEnumerable<T> PluginsAssignableTo<T>() where T : class =>
		from weakPlugin in _pluginMap
		let plugin = weakPlugin.Value.WeakToNullable()
		where plugin != null
		let castedPlugin = plugin.Instance.Instance as T
		where castedPlugin != null
		select castedPlugin;

	public IEnumerable<Namespaced<Driver>> AllDrivers() => GetAll(_driverMap);
	public IEnumerable<Namespaced<Driver>> DriversByPlugin(string pluginName) => GetByPlugin(_driverMap, pluginName);
	public Namespaced<Driver>? SpecificDriver(NamespacedName name) => GetSpecific(_driverMap, name);
	
	public IEnumerable<Namespaced<PluginAction>> AllPluginActions() => GetAll(_actionMap);
	public IEnumerable<Namespaced<PluginAction>> PluginActionsByPlugin(string pluginName) => 
		GetByPlugin(_actionMap, pluginName);
	public Namespaced<PluginAction>? SpecificPluginAction(NamespacedName name) => GetSpecific(_actionMap, name);
	
	public IEnumerable<Namespaced<PluginFunction>> AllPluginFunctions() => GetAll(_functionMap);
	public IEnumerable<Namespaced<PluginFunction>> PluginFunctionsByPlugin(string pluginName) => 
		GetByPlugin(_functionMap, pluginName);
	public Namespaced<PluginFunction>? SpecificPluginFunctions(NamespacedName name) => 
		GetSpecific(_functionMap, name);

	public IEnumerable<Namespaced<AsyncPluginAction>> AllAsyncPluginActions() => GetAll(_asyncActionMap);
	public IEnumerable<Namespaced<AsyncPluginAction>> AsyncPluginActionsByPlugin(string pluginName) => 
		GetByPlugin(_asyncActionMap, pluginName);
	public Namespaced<AsyncPluginAction>? SpecificAsyncPluginAction(NamespacedName name) =>
		GetSpecific(_asyncActionMap, name);

	public IEnumerable<Namespaced<AsyncPluginFunction>> AllAsyncPluginFunctions() => GetAll(_asyncFunctionMap);
	public IEnumerable<Namespaced<AsyncPluginFunction>> AsyncPluginFunctionsByPlugin(string pluginName) => 
		GetByPlugin(_asyncFunctionMap, pluginName);
	public Namespaced<AsyncPluginFunction>? SpecificAsyncPluginFunction(NamespacedName name) => 
		GetSpecific(_asyncFunctionMap, name);

	public IEnumerable<Namespaced<ScriptingSystem>> AllScriptingSystems() => GetAll(_scriptingSystemMap);
	public IEnumerable<Namespaced<ScriptingSystem>> ScriptingSystemsByPlugin(string pluginName) =>
		GetByPlugin(_scriptingSystemMap, pluginName);
	public Namespaced<ScriptingSystem>? SpecificScriptingSystem(NamespacedName name) => 
		GetSpecific(_scriptingSystemMap, name);
}