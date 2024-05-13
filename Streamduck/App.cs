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
using Streamduck.Actions;
using Streamduck.BaseFunctionality.Actions;
using Streamduck.BaseFunctionality.Triggers;
using Streamduck.Configuration;
using Streamduck.Configuration.Devices;
using Streamduck.Cores;
using Streamduck.Devices;
using Streamduck.Images;
using Streamduck.Plugins;
using Streamduck.Plugins.Extensions;
using Streamduck.Plugins.Loaders;
using Streamduck.Utils;

namespace Streamduck;

public class App : IStreamduck {
	private static readonly Logger _l = LogManager.GetCurrentClassLogger();

	private readonly List<NamespacedDeviceIdentifier> _discoveredDevices = new();

	private Config _config = null!;

	private bool _initialized;
	private bool _running;
	public static App? CurrentInstance { get; private set; }

	public PluginCollection? PluginCollection { get; private set; }

	public IReadOnlyList<NamespacedDeviceIdentifier> DiscoveredDeviceList {
		get {
			lock (_discoveredDevices) {
				return _discoveredDevices.AsEnumerable().ToList();
			}
		}
	}

	public ConcurrentDictionary<NamespacedDeviceIdentifier, Core> ConnectedDeviceList { get; } = new();

	public IPluginQuery Plugins => PluginCollection!;
	public IImageCollection Images { get; }
	public IReadOnlyCollection<NamespacedDeviceIdentifier> DiscoveredDevices => DiscoveredDeviceList;
	public IReadOnlyDictionary<NamespacedDeviceIdentifier, Core> ConnectedDevices => ConnectedDeviceList;


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

		// Load built-in plugin and external plugins
		var nameSet = new HashSet<string>();

		_l.Info("Scanning for core plugin...");
		var corePlugin = PluginLoader.Load(GetType().Assembly, nameSet)!;
		
		_l.Info(string.Join(", ", _config.PluginPaths));

		var foundPlugins = new[] { corePlugin }.AsEnumerable();

		foundPlugins = _config.PluginPaths.Aggregate(
			foundPlugins,
			(current, path) =>
				current.Concat(
					PluginLoader.LoadFromFolder(path, nameSet)
				)
		);

		PluginCollection = new PluginCollection(
			foundPlugins,
			_config
		);
		await PluginCollection.LoadAllPluginConfigs();

		await this.InvokePluginsLoaded();

		DeviceConnected += async (identifier, core) => await PluginCollection.InvokeDeviceConnected(identifier, core);
		DeviceDisconnected += async identifier => await PluginCollection.InvokeDeviceDisconnected(identifier);
		DeviceAppeared += async identifier => await PluginCollection.InvokeDeviceAppeared(identifier);
		DeviceDisappeared += async identifier => await PluginCollection.InvokeDeviceDisappeared(identifier);

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

	public async Task<bool> ConnectDevice(NamespacedDeviceIdentifier deviceIdentifier) {
		try {
			if (ConnectedDeviceList.ContainsKey(deviceIdentifier))
				throw new ApplicationException("Device is already connected");

			var driver = PluginCollection!.SpecificDriver(deviceIdentifier.NamespacedName);

			if (driver == null) {
				_l.Error("Driver '{}' wasn't found", deviceIdentifier.NamespacedName);
				return false;
			}

			var device = await driver.ConnectDevice(deviceIdentifier);
			device.Died += () => DeviceDisconnected?.Invoke(deviceIdentifier);
			var core = new CoreImpl(device, deviceIdentifier, PluginCollection!);

			lock (_discoveredDevices) {
				_discoveredDevices.Remove(deviceIdentifier);
			}

			if (!ConnectedDeviceList.TryAdd(deviceIdentifier, core))
				throw new ApplicationException("Couldn't add device, another connection was already made?");

			DeviceConnected?.Invoke(deviceIdentifier, core);

			if (await DeviceConfig.LoadConfig(core.DeviceIdentifier) is { } config)
				await core.LoadConfigIntoCore(config, PluginCollection);

			return true;
		} catch (Exception e) {
			_l.Error(e, "Failed to connect to device");
		}

		return false;
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
		foreach (var (identifier, _) in ConnectedDeviceList
			         .Where(k => !k.Value.IsAlive())) {
			ConnectedDeviceList.TryRemove(identifier, out var core);
			core?.Dispose();
		}

		_l.Debug("Checking all drivers for devices...");

		var _newDeviceList = new List<NamespacedDeviceIdentifier>();

		foreach (var driver in PluginCollection!.AllDrivers())
			_newDeviceList.AddRange(
				(await driver.ListDevices())
				.Where(device => !ConnectedDeviceList.ContainsKey(device))
			);

		lock (_discoveredDevices) {
			var newDevices = _newDeviceList
				.Where(device => !_discoveredDevices.Contains(device));

			var removedDevices = _discoveredDevices
				.Where(device => !_newDeviceList.Contains(device));

			foreach (var device in newDevices) DeviceAppeared?.Invoke(device);

			foreach (var device in removedDevices) DeviceDisappeared?.Invoke(device);

			_discoveredDevices.Clear();
			_discoveredDevices.AddRange(_newDeviceList);
			DeviceListRefreshed?.Invoke();
		}

		// Autoconnect
		foreach (var discoveredDevice in _newDeviceList
			         .Where(discoveredDevice => !ConnectedDeviceList.ContainsKey(discoveredDevice))
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
			foreach (var core in ConnectedDeviceList.Values)
				if (core is CoreImpl castedCore)
					castedCore.CallTick();

			var toWait = interval - (stopwatch.Elapsed.TotalSeconds - lastTime);

			if (toWait > 0.0) await Task.Delay(TimeSpan.FromSeconds(toWait));

			lastTime = stopwatch.Elapsed.TotalSeconds;
		}
	}
}