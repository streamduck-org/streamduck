// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

using System;
using System.Collections.Generic;
using System.Diagnostics.CodeAnalysis;
using System.Threading.Tasks;
using Streamduck.Actions;
using Streamduck.Attributes;
using Streamduck.Plugins;
using Streamduck.Plugins.Methods;

namespace StreamduckTest;

[TestFixture]
[SuppressMessage("Assertion", "NUnit2045:Use Assert.Multiple")]
public class PluginReflectorTest {
	private class TestPlugin : Plugin {
		public override string Name => "Test Plugin";

		public bool ActionWasCalled { get; private set; }
		public int Counter { get; private set; }

		[PluginMethod]
		public async Task TestAction() {
			await Task.Delay(3);
			ActionWasCalled = true;
		}

		[PluginMethod]
		public async Task TestOptionAction(TestOptions options) {
			await Task.Delay(1);
			options.Count++;
			Counter = options.Count;
		}

		[PluginMethod]
		public async Task TestConfigurableAction(TestOptions options, TestConfig config) {
			await Task.Delay(1);
			options.Count++;
			config.IncrementsPerformed++;
			Counter = options.Count;
		}

		public class TestOptions {
			public int Count { get; set; }
		}

		public class TestConfig {
			public int IncrementsPerformed { get; set; }
		}
	}

	[Test]
	public async Task TestActions() {
		var plugin = new TestPlugin();

		var methods = PluginReflector.GetMethods(plugin);
		using var actions = PluginReflector.AnalyzeActions(methods, plugin).GetEnumerator();

		{
			// Test Action
			var action = AnalyzeActionInfo(actions, "Test Action");

			await action.Invoke(await action.DefaultData());
			Assert.That(plugin.ActionWasCalled, Is.True, "Action was not properly called");
		}

		{
			// Test Action with option parameter
			var action = AnalyzeActionInfo(actions, "Test Option Action");

			var data = await action.DefaultData();
			Assert.That(action, Is.InstanceOf<ReflectedAction<TestPlugin.TestOptions>>(),
				"Action wasn't recognized as correct type");
			Assert.That(data, Is.InstanceOf<TestPlugin.TestOptions>(), "Default data wasn't the correct type");

			await action.Invoke(data);
			Assert.That(plugin.Counter, Is.EqualTo(1), "Action was not properly called or options didn't increment");

			await action.Invoke(data);
			Assert.That(plugin.Counter, Is.EqualTo(2), "Action was not properly called or options didn't increment");

			var castedData = (TestPlugin.TestOptions)data;
			Assert.That(castedData, Has.Count.EqualTo(2), "Action options weren't updated properly");
		}

		{
			// Test Configurable Action
			var action = AnalyzeActionInfo(actions, "Test Configurable Action");

			var data = await action.DefaultData();
			Assert.That(action,
				Is.InstanceOf<ConfigurableReflectedAction<TestPlugin.TestOptions, TestPlugin.TestConfig>>(),
				"Action wasn't recognized as correct type");
			Assert.That(data, Is.InstanceOf<TestPlugin.TestOptions>(), "Default data wasn't the correct type");

			await action.Invoke(data);
			Assert.That(plugin.Counter, Is.EqualTo(1), "Action was not properly called or options didn't increment");

			await action.Invoke(data);
			Assert.That(plugin.Counter, Is.EqualTo(2), "Action was not properly called or options didn't increment");

			var castedData = (TestPlugin.TestOptions)data;
			Assert.That(castedData, Has.Count.EqualTo(2), "Action options weren't updated properly");

			var configurable = (ConfigurableReflectedAction<TestPlugin.TestOptions, TestPlugin.TestConfig>)action;
			Assert.That(configurable.Config.IncrementsPerformed, Is.EqualTo(2),
				"Action config weren't maintained properly");
		}
	}

	private static PluginAction AnalyzeActionInfo(IEnumerator<PluginAction> enumerator, string name,
		string? description = null) {
		Console.WriteLine($"Testing action '{name}'");

		Assert.That(enumerator.MoveNext(), Is.True, $"Action '{name}' wasn't returned by reflector");
		var action = enumerator.Current;

		AssertInfo(action.Name, action.Description, name, description, "Action");

		return action;
	}

	private static void AssertInfo(string actualName, string? actualDescription, string name, string? description,
		string logPrefix) {
		Assert.That(actualName, Is.EqualTo(name), $"{logPrefix} '{name}' had invalid title");

		if (description != null)
			Assert.That(actualDescription, Is.EqualTo(description), $"{logPrefix} '{name}' had invalid description");
	}
}