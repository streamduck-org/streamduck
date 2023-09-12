using System;
using System.Collections.Concurrent;
using System.Collections.Generic;
using System.Linq;
using System.Runtime.InteropServices.ComTypes;
using System.Threading;
using System.Threading.Tasks;
using NLog;
using SixLabors.ImageSharp;
using Streamduck.Configuration;
using Streamduck.Definitions;
using Streamduck.Definitions.Api;
using Streamduck.Definitions.Devices;
using Streamduck.Definitions.Inputs;
using Streamduck.Plugins;
using Streamduck.Plugins.Loaders;

namespace Streamduck;

public class App {
	private static readonly Logger L = LogManager.GetCurrentClassLogger();

	private readonly List<NamespacedDeviceIdentifier> _discoveredDevices = new();
	private readonly ConcurrentDictionary<NamespacedDeviceIdentifier, Core> _connectedDevices = new ();
	
	private Config? _config;

	private ConcurrentDictionary<NamespacedName, WeakReference<WrappedDriver>> _driverMap = new();

	private bool _initialized;
	private ConcurrentDictionary<string, WeakReference<WrappedPlugin>> _pluginMap = new();
	private PluginAssembly[] _plugins = Array.Empty<PluginAssembly>();
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
		_config = await Config.Get();

		_initialized = true;
	}

	public void Unload(WrappedPlugin plugin) {
		var assembly = _plugins.Single(plugin.BelongsTo);
		_plugins = _plugins.Where(a => !a.Equals(assembly)).ToArray();
		assembly.Unload();
	}


	/**
	 * Runs the Streamduck software
	 */
	public async Task Run(CancellationTokenSource cts) {
		if (!_initialized) throw new ApplicationException("Init method was not called");

		_running = true;
		cts.Token.Register(() => _running = false);

		await DeviceDiscoverTask(cts);
	}

	public async Task ConnectDevice(NamespacedDeviceIdentifier deviceIdentifier) {
		try {
			if (_connectedDevices.ContainsKey(deviceIdentifier)) {
				throw new ApplicationException("Device is already connected");
			}
			
			var driver = GetDriver(deviceIdentifier.NamespacedName);

			if (driver == null) {
				L.Error("Driver '{}' wasn't found", deviceIdentifier.NamespacedName);
				return;
			}

			var device = await driver.ConnectDevice(deviceIdentifier);
			var core = new CoreImpl(device, deviceIdentifier);
			if (!_connectedDevices.TryAdd(deviceIdentifier, core)) {
				throw new ApplicationException("Couldn't add device, another connection was already made?");
			}
		} catch (Exception e) {
			L.Error(e, "Failed to connect to device");
		}
	}

	private async Task DeviceDiscoverTask(CancellationTokenSource cts) {
		while (_running) {
			L.Debug("Cleaning up dead devices...");
			foreach (var (identifier, _) in _connectedDevices
				         .Where(k => !k.Value.IsAlive())) {
				_connectedDevices.TryRemove(identifier, out var core);
				core?.Dispose();
			}
			
			L.Debug("Checking all drivers for devices...");
			_discoveredDevices.Clear();

			foreach (var driver in Drivers()) {
				_discoveredDevices.AddRange(await driver.ListDevices());
			}

			// Autoconnect
			foreach (var discoveredDevice in _discoveredDevices
				         .Where(discoveredDevice => !_connectedDevices.ContainsKey(discoveredDevice))
				         .Where(discoveredDevice => _config!.AutoconnectDevices.Contains(discoveredDevice))) {
				L.Info("Autoconnecting to {}", discoveredDevice);
				await ConnectDevice(discoveredDevice);
			}

			await Task.Delay(TimeSpan.FromSeconds(_config!.DeviceCheckDelay), cts.Token);
		}
	}
}