using System;
using System.Collections.Concurrent;
using System.Collections.Generic;
using System.Linq;
using System.Threading.Tasks;
using NLog;
using SixLabors.ImageSharp;
using Streamduck.Configuration;
using Streamduck.Definitions.Devices;
using Streamduck.Definitions.Inputs;
using Streamduck.Plugins;
using Streamduck.Plugins.Loaders;

namespace Streamduck;

public class App {
	private static readonly Logger L = LogManager.GetCurrentClassLogger();

	private readonly List<NamespacedDeviceIdentifier> _discoveredDevices = new();
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
	public async Task Run() {
		if (!_initialized) throw new ApplicationException("Init method was not called");

		_running = true;

		await Task.Run(DeviceDiscoverTask);
	}

	private async Task DeviceDiscoverTask() {
		while (_running) {
			L.Debug("Checking all drivers for devices...");
			_discoveredDevices.Clear();

			// foreach (var driver in Drivers()) {
			// 	_discoveredDevices.AddRange(await driver.ListDevices());
			// }

			var firstDriver = Drivers().First();
			var firstDeviceIdentifier = (await firstDriver.ListDevices()).First();
			var firstDevice = await firstDriver.ConnectDevice(firstDeviceIdentifier);

			var catKey = 1;
			var dogKey = 2;

			for (var i = 0; i < firstDevice.Inputs.Length; i++) {
				var input = firstDevice.Inputs[i];

				if (input is IInputButton button) {
					var captured = i;
					button.ButtonPressed += () => L.Info("{} pressed", captured);
					button.ButtonReleased += () => L.Info("{} released", captured);

					if (input is IInputDisplay display) {
						button.ButtonPressed += async () => {
							var appended = display.AppendHashKey(catKey);
							if (await display.ApplyImage(appended)) return;
							using var cat = await Image.LoadAsync("cat-1285634_1920.png");
							await display.UploadImage(appended, cat);
							await display.ApplyImage(appended);
						};
						button.ButtonReleased += async () => {
							var appended = display.AppendHashKey(dogKey);
							if (await display.ApplyImage(appended)) return;
							using var dog = await Image.LoadAsync("download.jpeg");
							await display.UploadImage(appended, dog);
							await display.ApplyImage(appended);
						};
					}
				}
				
				if (input is IInputEncoder encoder) {
					var captured = i;
					encoder.EncoderTwisted += val => L.Info("{} twisted {}", captured, val);
				}
				
				if (input is IInputTouchScreen touchScreen) {
					var captured = i;
					touchScreen.TouchScreenPressed += pos => L.Info("{} pressed at {}", captured, pos);
					touchScreen.TouchScreenReleased += pos => L.Info("{} released at {}", captured, pos);
				}
				
				if (input is IInputTouchScreen.Drag drag) {
					var captured = i;
					drag.TouchScreenDragStart += pos => L.Info("{} drag start at {}", captured, pos);
					drag.TouchScreenDragEnd += pos => L.Info("{} drag end at {}", captured, pos);
				}
			}

			await Task.Delay(TimeSpan.FromSeconds(50000));
		}
	}
}