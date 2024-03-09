// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

using System;
using System.Collections.Concurrent;
using System.Collections.Generic;
using System.Diagnostics;
using System.IO;
using System.Linq;
using System.Threading;
using System.Threading.Tasks;
using NLog;
using Streamduck.Configuration;
using Streamduck.Cores;
using Streamduck.Devices;
using Streamduck.Plugins;
using Streamduck.Plugins.Extensions;
using Streamduck.Plugins.Loaders;

namespace Streamduck;

public class App {
	public static App? CurrentInstance { get; private set; }
	
	private static readonly Logger _l = LogManager.GetCurrentClassLogger();

	private readonly List<NamespacedDeviceIdentifier> _discoveredDevices = new();

	private Config _config = null!;

	private bool _initialized;
	private bool _running;

	public PluginCollection? Plugins { get; private set; }

	public IReadOnlyList<NamespacedDeviceIdentifier> DiscoveredDevices {
		get {
			lock (_discoveredDevices) {
				return _discoveredDevices.AsEnumerable().ToList();
			}
		}
	}

	public ConcurrentDictionary<NamespacedDeviceIdentifier, Core> ConnectedDevices { get; } = new();
	public event Action? DeviceListRefreshed;

	/**
	 * Device is connected to Streamduck
	 */
	public event Action<NamespacedDeviceIdentifier, Core>? DeviceConnected;

	/**
	 * Device is disconnected from Streamduck
	 */
	public event Action<NamespacedDeviceIdentifier>? DeviceDisconnected;

	/**
	 * Device is discovered by a driver
	 */
	public event Action<NamespacedDeviceIdentifier>? DeviceAppeared;

	/**
	 * Device is no longer available
	 */
	public event Action<NamespacedDeviceIdentifier>? DeviceDisappeared;

	/**
	 * Initializes Streamduck (eg. load plugins, load auto-connects)
	 */
	public async Task Init() {
		if (_initialized) throw new ApplicationException("App was already initialized");

		_config = await Config.Get();

		Directory.CreateDirectory("plugins");
		
		// Load built-in plugin and external plugins
		var nameSet = new HashSet<string>();

		_l.Info("Scanning for core plugin...");
		var corePlugin = PluginLoader.Load(GetType().Assembly, nameSet)!;
		
		Plugins = new PluginCollection(new []{ corePlugin }
				.Concat(PluginLoader.LoadFromFolder("plugins", nameSet)), _config);
		await Plugins.LoadAllPluginConfigs();

		await Plugins.InvokePluginsLoaded();

		DeviceConnected += async (identifier, core) => await Plugins.InvokeDeviceConnected(identifier, core);
		DeviceDisconnected += async identifier => await Plugins.InvokeDeviceDisconnected(identifier);

		_initialized = true;
		CurrentInstance = this;
	}


	/**
	 * Runs the Streamduck software
	 */
	public Task Run(CancellationTokenSource cts) {
		if (!_initialized) throw new ApplicationException("Init method was not called");

		_running = true;
		cts.Token.Register(() => _running = false);

		return Task.WhenAll(DeviceDiscoverTask(cts), TickTask(cts));
	}

	public async Task ConnectDevice(NamespacedDeviceIdentifier deviceIdentifier) {
		try {
			if (ConnectedDevices.ContainsKey(deviceIdentifier))
				throw new ApplicationException("Device is already connected");

			var driver = Plugins!.SpecificDriver(deviceIdentifier.NamespacedName);

			if (driver == null) {
				_l.Error("Driver '{}' wasn't found", deviceIdentifier.NamespacedName);
				return;
			}

			var device = await driver.ConnectDevice(deviceIdentifier);
			device.Died += () => DeviceDisconnected?.Invoke(deviceIdentifier);
			var core = new CoreImpl(device, deviceIdentifier, Plugins!);

			lock (_discoveredDevices) {
				_discoveredDevices.Remove(deviceIdentifier);
			}

			if (!ConnectedDevices.TryAdd(deviceIdentifier, core))
				throw new ApplicationException("Couldn't add device, another connection was already made?");

			DeviceConnected?.Invoke(deviceIdentifier, core);
		} catch (Exception e) {
			_l.Error(e, "Failed to connect to device");
		}
	}

	private async Task DeviceDiscoverTask(CancellationTokenSource cts) {
		await RefreshDevices();
		while (_running) {
			await Task.Delay(TimeSpan.FromSeconds(_config!.DeviceCheckDelay), cts.Token);
			await RefreshDevices();
		}
	}

	public async Task RefreshDevices() {
		_l.Debug("Cleaning up dead devices...");
		foreach (var (identifier, _) in ConnectedDevices
			         .Where(k => !k.Value.IsAlive())) {
			ConnectedDevices.TryRemove(identifier, out var core);
			core?.Dispose();
		}

		_l.Debug("Checking all drivers for devices...");

		var _newDeviceList = new List<NamespacedDeviceIdentifier>();

		foreach (var driver in Plugins!.AllDrivers()) {
			_newDeviceList.AddRange((await driver.ListDevices())
				.Where(device => !ConnectedDevices.ContainsKey(device)));
		}

		lock (_discoveredDevices) {
			var newDevices = _newDeviceList
				.Where(device => !_discoveredDevices.Contains(device));

			var removedDevices = _discoveredDevices
				.Where(device => !_newDeviceList.Contains(device));

			foreach (var device in newDevices) {
				DeviceAppeared?.Invoke(device);
			}

			foreach (var device in removedDevices) {
				DeviceDisappeared?.Invoke(device);
			}

			_discoveredDevices.Clear();
			_discoveredDevices.AddRange(_newDeviceList);
			DeviceListRefreshed?.Invoke();
		}

		// Autoconnect
		foreach (var discoveredDevice in _newDeviceList
			         .Where(discoveredDevice => !ConnectedDevices.ContainsKey(discoveredDevice))
			         .Where(discoveredDevice => _config.AutoconnectDevices.Contains(discoveredDevice))) {
			_l.Info("Autoconnecting to {}", discoveredDevice);
			await ConnectDevice(discoveredDevice);
		}
	}

	private async Task TickTask(CancellationTokenSource cts) {
		var stopwatch = Stopwatch.StartNew();

		var lastTime = stopwatch.Elapsed.TotalSeconds;
		var interval = 1.0 / _config.TickRate;

		while (_running) {
			foreach (var core in ConnectedDevices.Values) {
				if (core is CoreImpl castedCore) castedCore.CallTick();
			}

			var toWait = interval - (stopwatch.Elapsed.TotalSeconds - lastTime);

			if (toWait > 0.0) await Task.Delay(TimeSpan.FromSeconds(toWait));

			lastTime = stopwatch.Elapsed.TotalSeconds;
		}
	}
}