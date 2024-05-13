// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

using System;
using System.Collections.Generic;
using System.IO;
using System.Text.Json;
using System.Text.Json.Serialization;
using System.Threading.Tasks;
using NLog;
using Streamduck.Devices;
using Streamduck.Plugins;

namespace Streamduck.Configuration;

/**
 * Configuration for Streamduck
 */
public class Config {
	public const string StreamduckFolderName = "streamduck";
	public const string ConfigFileName = "config.json";

	public static readonly string StreamduckFolder = Path.Join(
		Environment.GetFolderPath(Environment.SpecialFolder.ApplicationData),
		StreamduckFolderName
	);

	private static readonly Logger L = LogManager.GetCurrentClassLogger();

	private static Config? _configInstance;

	/**
	 * How often to tick all the button scripts, in hz
	 */
	public double TickRate { get; set; } = 60.0;

	/**
	 * How often to attempt to render every button script, in frames/second
	 */
	public double FrameRate { get; set; } = 60.0;

	/**
	 * How long to wait between checking for new devices from all loaded drivers
	 */
	public double DeviceCheckDelay { get; set; } = 30.0;

	/**
	 * Let the websocket connection listen for 0.0.0.0 (open to internet)
	 */
	public bool OpenToInternet { get; set; }

	/**
	 * Websocket port to use, defaults to 57234
	 */
	public int WebSocketPort { get; set; } = 42131;

	/**
	 * List of paths Streamduck will check for plugins
	 */
	public List<string> PluginPaths { get; set; } = ["plugins"];

	/**
	 * Devices that should be automatically connected to
	 */
	[JsonInclude]
	public HashSet<NamespacedDeviceIdentifier> AutoconnectDevices { get; private set; } = [];

	public static NamespacedName DefaultRendererName { get; }
		= new("Streamduck Core Plugin", "Default Renderer");

	/**
	 * Default renderer to be used for new screen items
	 */
	public NamespacedName DefaultRenderer { get; set; } = DefaultRendererName;

	public async Task AddDeviceToAutoconnect(NamespacedDeviceIdentifier deviceIdentifier) {
		lock (AutoconnectDevices) {
			AutoconnectDevices.Add(deviceIdentifier);
		}

		L.Info("Added {} to autoconnect", deviceIdentifier.DeviceIdentifier);

		await SaveConfig();
	}

	public async Task RemoveDeviceFromAutoconnect(NamespacedDeviceIdentifier deviceIdentifier) {
		lock (AutoconnectDevices) {
			AutoconnectDevices.Remove(deviceIdentifier);
		}

		L.Info("Removed {} from autoconnect", deviceIdentifier.DeviceIdentifier);

		await SaveConfig();
	}

	private static async Task<Config> _loadConfig() {
		var path = Path.Join(
			StreamduckFolder,
			ConfigFileName
		);

		L.Info("Loading config...");

		if (File.Exists(path)) {
			var content = await File.ReadAllBytesAsync(path);

			L.Debug("Trying to read existing config...");

			try {
				using var memoryStream = new MemoryStream(content);

				var deserializedConfig = await JsonSerializer.DeserializeAsync<Config>(
					memoryStream
				);

				if (deserializedConfig != null) return deserializedConfig;
			} catch (Exception e) {
				L.Error("Error happened while trying to load config {0}", e);
				// TODO: Backup invalid config
			}
		}

		L.Debug("No config found, creating new one...");
		var config = new Config();

		await config.SaveConfig();

		return config;
	}

	/**
	 * Saves config to json file in app data
	 */
	public async Task SaveConfig() {
		try {
			Directory.CreateDirectory(StreamduckFolder);
		} catch (Exception e) {
			L.Error("Error happened while trying to create folders for config {0}", e);
			return;
		}

		var path = Path.Join(
			StreamduckFolder,
			ConfigFileName
		);

		try {
			using var buffer = new MemoryStream();

			await JsonSerializer.SerializeAsync(
				buffer,
				this
			);

			Console.WriteLine(path);

			L.Info("Saving app config...");
			await File.WriteAllBytesAsync(path, buffer.ToArray());
			L.Info("Saved app config");
		} catch (Exception e) {
			L.Error("Error happened while trying to save config {0}", e);
		}
	}

	/**
	 * If config wasn't loaded yet, loads config from json file.
	 * If file doesn't exist, creates a default AppConfig and saves it.
	 * If config is already loaded, provides that config instance
	 */
	public static async Task<Config> Get() {
		_configInstance ??= await _loadConfig();
		return _configInstance;
	}

	public static Config? IgnorantGet() => _configInstance;
}