using System;
using System.Collections.Generic;
using System.Linq;
using System.Threading.Tasks;
using ElgatoStreamDeck;
using Streamduck.Definitions.Devices;
using Streamduck.Plugins;
using Device = Streamduck.Definitions.Devices.Device;
using ElgatoDevice = ElgatoStreamDeck.Device;

namespace StreamduckStreamDeck;

public class StreamDeckDriver : Driver {
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

	private readonly DeviceManager _manager;

	public StreamDeckDriver(DeviceManager manager) {
		_manager = manager;
	}

	public override string Name => "StreamDeckDriver";

	public override Task<IEnumerable<DeviceIdentifier>> ListDevices() => Task.FromResult(_manager.ListDevices()
		.Where(t => IsValid(t.Item1))
		.Select(t => new DeviceIdentifier(t.Item2, KindToDescription(t.Item1)))
	);

	public override Task<Device> ConnectDevice(DeviceIdentifier identifier) {
		var device = _manager.ConnectDeviceConcurrent(DescriptionToKind(identifier.Description), identifier.Identifier);
		return Task.FromResult(new StreamDeckDevice(device, identifier) as Device);
	}

	private static bool IsValid(Kind kind) => kind != Kind.Unknown;

	internal static string KindToDescription(Kind kind) => kind switch {
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

	internal static Kind DescriptionToKind(string desc) => desc switch {
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