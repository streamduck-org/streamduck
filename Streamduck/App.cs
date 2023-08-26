using System;
using System.Collections.Concurrent;
using System.Collections.Generic;
using System.Linq;
using System.Threading.Tasks;
using NLog;
using Streamduck.Configuration;
using Streamduck.Definitions.Devices;
using Streamduck.Plugins;
using Streamduck.Plugins.Loaders;

namespace Streamduck;

public class App {
	private static readonly Logger L = LogManager.GetCurrentClassLogger();
    
	private readonly List<NamespacedDeviceIdentifier>  _discoveredDevices = new();
	
	private ConcurrentDictionary<NamespacedName, WeakReference<WrappedDriver>> _driverMap = new();
	private ConcurrentDictionary<string, WeakReference<WrappedPlugin>> _pluginMap = new();
	private PluginAssembly[] _plugins = Array.Empty<PluginAssembly>();
	private Config _config;

	private bool _running;

	public IEnumerable<WrappedPlugin> Plugins() => _pluginMap
		.Select(k => {
			k.Value.TryGetTarget(out var v);
			return v;
		})
		.Where(t => t != null)
		.Select(t => t!);

	public IEnumerable<WrappedDriver> Drivers() => _driverMap
		.Select(k => {
			k.Value.TryGetTarget(out var v);
			return v;
		})
		.Where(t => t != null)
		.Select(t => t!);

	public WrappedPlugin? GetPlugin(string name) {
		if (!_pluginMap.ContainsKey(name)) return null;
		_pluginMap[name].TryGetTarget(out var plugin);
		return plugin;
	}

	public WrappedDriver? GetDriver(NamespacedName name) {
		if (!_driverMap.ContainsKey(name)) return null;
		_driverMap[name].TryGetTarget(out var driver);
		return driver;
	}

	/**
	 * Initializes Streamduck (eg. load plugins, load auto-connects)
	 */
	public async Task Init() {
		_plugins = PluginLoader.LoadFromFolder("plugins").ToArray();
		_pluginMap = new ConcurrentDictionary<string, WeakReference<WrappedPlugin>>(
			_plugins
				.SelectMany(a => a.Plugins)
				.ToDictionary(p => p.Name, p => new WeakReference<WrappedPlugin>(p))
		);
		_driverMap = new ConcurrentDictionary<NamespacedName, WeakReference<WrappedDriver>>(
			_plugins
				.SelectMany(a => a.Plugins)
				.SelectMany(p => p.Drivers)
				.ToDictionary(d => d.Name, d => new WeakReference<WrappedDriver>(d))
		);
		_config = await Config.GetConfig();
	}

	public void Unload(WrappedPlugin plugin) {
		var assembly = _plugins.Single(plugin.BelongsTo);
		_plugins = _plugins.Where(a => !a.Equals(assembly)).ToArray();
		assembly.Unload();
	}


	/**
	 * Runs the Streamduck software
	 */
	public async Task Run() {
		_running = true;

		await Task.Run(DeviceDiscoverTask);
	}
	private async Task DeviceDiscoverTask() {
		while (_running) {
			L.Debug("Checking all drivers for devices...");
			_discoveredDevices.Clear();
			
			foreach (var driver in Drivers()) {
				_discoveredDevices.AddRange(await driver.ListDevices());
			}

			foreach (var device in _discoveredDevices) {
				L.Info("Got {}", device);
			}

			await Task.Delay(TimeSpan.FromSeconds(_config.DeviceCheckDelay));
		}
	}
}