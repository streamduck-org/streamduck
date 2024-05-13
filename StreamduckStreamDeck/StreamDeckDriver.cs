// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

using System.Collections.Generic;
using System.Linq;
using System.Threading.Tasks;
using ElgatoStreamDeck;
using Streamduck.Devices;
using Streamduck.Interfaces;
using Streamduck.Plugins;
using Device = Streamduck.Devices.Device;

namespace StreamduckStreamDeck;

public class StreamDeckDriver(DeviceManager manager) : Driver, IConfigurable<StreamDeckDeviceOptions> {
	private const string StreamDeckOriginalDesc = "Stream Deck Original";
	private const string StreamDeckOriginalV2Desc = "Stream Deck Original V2";
	private const string StreamDeckMiniDesc = "Stream Deck Mini";
	private const string StreamDeckXlDesc = "Stream Deck XL";
	private const string StreamDeckXlV2Desc = "Stream Deck XL V2";
	private const string StreamDeckMk2Desc = "Stream Deck Mk2";
	private const string StreamDeckMiniMk2Desc = "Stream Deck Mini Mk2";
	private const string StreamDeckPedalDesc = "Stream Deck Pedal";
	private const string StreamDeckPlusDesc = "Stream Deck Plus";
	private const string StreamDeckUnknownDesc = "Unknown";

	public override string Name => "Stream Deck Driver";

	public StreamDeckDeviceOptions Config { get; set; } = new();

	public override Task<IEnumerable<DeviceIdentifier>> ListDevices() {
		return Task.FromResult(
			manager.ListDevices()
				.Where(t => IsValid(t.Item1))
				.Select(t => new DeviceIdentifier(t.Item2, KindToDescription(t.Item1)))
		);
	}

	public override Task<Device> ConnectDevice(DeviceIdentifier identifier) {
		var device = manager.ConnectDeviceConcurrent(DescriptionToKind(identifier.Description), identifier.Identifier);
		return Task.FromResult(new StreamDeckDevice(device, identifier) as Device);
	}

	private static bool IsValid(Kind kind) => kind != Kind.Unknown;

	internal static string KindToDescription(Kind kind) {
		return kind switch {
			Kind.Original => StreamDeckOriginalDesc,
			Kind.OriginalV2 => StreamDeckOriginalV2Desc,
			Kind.Mini => StreamDeckMiniDesc,
			Kind.Xl => StreamDeckXlDesc,
			Kind.XlV2 => StreamDeckXlV2Desc,
			Kind.Mk2 => StreamDeckMk2Desc,
			Kind.MiniMk2 => StreamDeckMiniMk2Desc,
			Kind.Pedal => StreamDeckPedalDesc,
			Kind.Plus => StreamDeckPlusDesc,
			_ => StreamDeckUnknownDesc
		};
	}

	internal static Kind DescriptionToKind(string desc) {
		return desc switch {
			StreamDeckOriginalDesc => Kind.Original,
			StreamDeckOriginalV2Desc => Kind.OriginalV2,
			StreamDeckMiniDesc => Kind.Mini,
			StreamDeckXlDesc => Kind.Xl,
			StreamDeckXlV2Desc => Kind.XlV2,
			StreamDeckMk2Desc => Kind.Mk2,
			StreamDeckMiniMk2Desc => Kind.MiniMk2,
			StreamDeckPedalDesc => Kind.Pedal,
			StreamDeckPlusDesc => Kind.Plus,
			_ => Kind.Unknown
		};
	}
}