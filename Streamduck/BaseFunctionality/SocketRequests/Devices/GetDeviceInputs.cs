// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

using System.Linq;
using System.Text.Json.Serialization;
using System.Threading.Tasks;
using Streamduck.Attributes;
using Streamduck.Devices;
using Streamduck.Inputs;
using Streamduck.Socket;

namespace Streamduck.BaseFunctionality.SocketRequests.Devices;

[AutoAdd]
public class GetDeviceInputs : SocketRequest<GetDeviceInputs.Request> {
	public class Request {
		public required NamespacedDeviceIdentifier Identifier { get; set; }
	}

	public class InputData(Input input) {
		public int X { get; set; } = input.X;
		public int Y { get; set; } = input.Y;
		public uint W { get; set; } = input.W;
		public uint H { get; set; } = input.H;
		[JsonConverter(typeof(JsonStringEnumConverter<InputIcon>))]
		public InputIcon Icon { get; set; } = input.Icon;
	}

	public override string Name => "Get Device Inputs";

	public override Task Received(SocketRequester request, Request data) {
		if (!SRUtil.GetDevice(request, data.Identifier, out var device)) return Task.CompletedTask;

		request.SendBack(device.Inputs.Select(i => new InputData(i)));
		return Task.CompletedTask;
	}
}