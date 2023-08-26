using System.Collections.Generic;
using System.Linq;
using System.Threading.Tasks;
using ElgatoStreamDeck;
using Streamduck.Definitions.Devices;
using Streamduck.Definitions.Inputs;
using Streamduck.Plugins;
using Input = Streamduck.Definitions.Inputs.Input;

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
		.Select(t => new DeviceIdentifier {
			Identifier = t.Item2,
			Description = KindToDescription(t.Item1)
		}));

	public override ValueTask<DeviceMetadata?> DescribeDevice(DeviceIdentifier identifier) => 
		ValueTask.FromResult(KindToMetadata(DescriptionToKind(identifier.Description)));

	private static bool IsValid(Kind kind) => kind != Kind.Unknown;

	private static string KindToDescription(Kind kind) => kind switch {
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

	private static Kind DescriptionToKind(string desc) => desc switch {
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

	private static IEnumerable<Input> ButtonInputs(Kind kind) {
		var columnCount = kind.ColumnCount();

		for (var i = 0; i < kind.KeyCount(); i++) {
			var x = i % columnCount;
			var y = i / columnCount;

			yield return new Input {
				X = x,
				Y = y,
				W = 1,
				H = 1,
				Icon = InputIcon.Button,
				Behaviors = new[] { InputBehavior.Button },
				Resolution = kind == Kind.Pedal ? null : kind.KeyImageMode().Resolution
			};
		}
	}

	private static IEnumerable<Input> PlusInputs(Kind kind) {
		yield return new Input {
			X = 0,
			Y = 2,
			W = 4,
			H = 1,
			Icon = InputIcon.TouchScreen,
			Behaviors = new[] { InputBehavior.TouchScreen },
			Resolution = kind.LcdStripSize()!
		};

		for (var i = 0; i < kind.EncoderCount(); i++) {
			yield return new Input {
				X = i,
				Y = 3,
				W = 1,
				H = 1,
				Icon = InputIcon.Encoder,
				Behaviors = new[] { InputBehavior.Button, InputBehavior.Encoder },
				Resolution = null
			};
		}
	}

	private static DeviceMetadata? KindToMetadata(Kind kind) => kind switch {
		Kind.Original => new DeviceMetadata {
			Inputs = ButtonInputs(kind).ToArray()
		},
		Kind.OriginalV2 => new DeviceMetadata {
			Inputs = ButtonInputs(kind).ToArray()
		},
		Kind.Mini => new DeviceMetadata {
			Inputs = ButtonInputs(kind).ToArray()
		},
		Kind.Xl => new DeviceMetadata {
			Inputs = ButtonInputs(kind).ToArray()
		},
		Kind.XlV2 => new DeviceMetadata {
			Inputs = ButtonInputs(kind).ToArray()
		},
		Kind.Mk2 => new DeviceMetadata {
			Inputs = ButtonInputs(kind).ToArray()
		},
		Kind.MiniMk2 => new DeviceMetadata {
			Inputs = ButtonInputs(kind).ToArray()
		},
		Kind.Pedal => new DeviceMetadata {
			Inputs = ButtonInputs(kind).ToArray()
		},
		Kind.Plus => new DeviceMetadata {
			Inputs = ButtonInputs(kind)
				.Concat(PlusInputs(kind))
				.ToArray()
		},
		_ => null
	};
}