using System;
using System.Collections.Generic;
using System.IO;
using System.Text.Json;
using System.Text.Json.Serialization;
using System.Threading.Tasks;
using NLog;
using Streamduck.Definitions.Devices;

namespace Streamduck.Configuration;

/**
 * Configuration for Streamduck
 */
public class Config {
	private const string StreamduckFolderName = "streamduck";
	private const string ConfigFileName = "config.json";

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

	/*
	 * How long to wait between checking for new devices from all loaded drivers
	 */
	public double DeviceCheckDelay { get; set; } = 30.0;

	/**
	 * Devices that should be automatically connected to
	 */
	[JsonInclude]
	public HashSet<NamespacedDeviceIdentifier> AutoconnectDevices { get; private set; } = new();

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
			Environment.GetFolderPath(Environment.SpecialFolder.ApplicationData),
			StreamduckFolderName,
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
		var folderPath = Path.Join(
			Environment.GetFolderPath(Environment.SpecialFolder.ApplicationData),
			StreamduckFolderName
		);

		try {
			Directory.CreateDirectory(folderPath);
		} catch (Exception e) {
			L.Error("Error happened while trying to create folders for config {0}", e);
			return;
		}

		var path = Path.Join(
			folderPath,
			ConfigFileName
		);

		try {
			using var buffer = new MemoryStream();

			await JsonSerializer.SerializeAsync(
				buffer,
				this
			);

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