// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

using System;
using System.Threading.Tasks;
using Streamduck.Actions;
using Streamduck.Attributes;

namespace Streamduck.BaseFunctionality.Actions;

[AutoAdd]
public class TestAction : PluginAction<TestAction.Data> {
	public class Data {
		public string? Text { get; set; } 
	}
	public override string Name => "Test Action";
	public override string? Description => "Runs some random stuff";
	public override Task Invoke(Data data) {
		if (data.Text is { } text) Console.WriteLine($"Button is '{text}'");
		return Task.CompletedTask;
	}
}