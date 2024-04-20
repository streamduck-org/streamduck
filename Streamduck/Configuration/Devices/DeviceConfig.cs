// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

using System;
using System.IO;
using System.Text.Json;
using System.Text.Json.Serialization;
using System.Threading.Tasks;
using NLog;
using Streamduck.Devices;

namespace Streamduck.Configuration.Devices;

public class DeviceConfig {
	private static readonly Logger _l = LogManager.GetCurrentClassLogger();
	
	public NamespacedDeviceIdentifier Device { get; set; }
	public SerializedScreen[] ScreenStack { get; set; } = [];

	public static readonly string DeviceConfigFolder = Path.Join(
		Config.StreamduckFolder,
		"devices"
	);

	private static string DeviceFolderPath(NamespacedDeviceIdentifier deviceIdentifier) =>
		Path.Join(
			DeviceConfigFolder,
			deviceIdentifier.PluginName,
			deviceIdentifier.DriverName,
			deviceIdentifier.Description
		);

	private static string DeviceFilePath(NamespacedDeviceIdentifier deviceIdentifier) =>
		Path.Join(
			DeviceFolderPath(deviceIdentifier),
			$"{deviceIdentifier.Identifier}.json"
		);
	
	private static string DeviceFilePath(NamespacedDeviceIdentifier deviceIdentifier, string folderPath) =>
		Path.Join(
			folderPath,
			$"{deviceIdentifier.Identifier}.json"
		);

	public async Task SaveConfig() {
		var folder = DeviceFolderPath(Device);
		
		try {
			Directory.CreateDirectory(folder);
		} catch (Exception e) {
			_l.Error("Error happened while trying to create folders for config {0}", e);
			return;
		}

		var jsonData = JsonSerializer.Serialize(this);
		var filePath = DeviceFilePath(Device, folder);

		await File.WriteAllTextAsync(filePath, jsonData);
	}

	private static readonly JsonSerializerOptions _jsonSerializerOptions = new JsonSerializerOptions {
		UnmappedMemberHandling = JsonUnmappedMemberHandling.Disallow
	};

	public static async Task<DeviceConfig?> LoadConfig(NamespacedDeviceIdentifier deviceIdentifier) {
		var filePath = DeviceFilePath(deviceIdentifier);

		if (!File.Exists(filePath)) {
			_l.Error($"Failed to locate device config at '{filePath}'");
			return null;
		}

		var stringData = await File.ReadAllTextAsync(filePath);

		try {
			var data = JsonSerializer.Deserialize<DeviceConfig>(stringData, _jsonSerializerOptions);
			return data;
		} catch (JsonException e) {
			_l.Error($"Failed to load device config at '{filePath}':\n{e.Message}");
			return null;
		}
	}
}