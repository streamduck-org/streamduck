using System;
using System.Collections.Generic;
using System.Diagnostics.CodeAnalysis;
using System.Threading.Tasks;
using Streamduck.Attributes;
using Streamduck.Plugins;
using Streamduck.Plugins.Methods;
using Desc = Streamduck.Attributes.DescriptionAttribute;

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

		public class TestOptions {
			public int Count { get; set; }
		}

		[PluginMethod]
		public async Task TestConfigurableAction(TestOptions options) {
			await Task.Delay(1);
			options.Count++;
			Counter = options.Count;
		}
	}

	[Test]
	public async Task TestActions() {
		var plugin = new TestPlugin();
		
		var methods = PluginReflector.GetMethods(plugin);
		using var actions = PluginReflector.AnalyzeActions(methods, plugin).GetEnumerator();
		
		{ // Test Action
			var action = AnalyzeActionInfo(actions, "Test Action");
			
			await action.Invoke();
			Assert.That(plugin.ActionWasCalled, Is.True, "Action was not properly called");
		}
		
		{ // Test Configurable Action
			var action = AnalyzeActionInfo(actions, "Test Configurable Action");
			
			Assert.That(action, Is.InstanceOf<ConfigurableReflectedAction<TestPlugin.TestOptions>>(), "Action wasn't recognized as configurable");
			
			await action.Invoke();
			Assert.That(plugin.Counter, Is.EqualTo(1), "Action was not properly called or options didn't increment");
			
			await action.Invoke();
			Assert.That(plugin.Counter, Is.EqualTo(2), "Action was not properly called or options didn't increment");

			var configurable = (ConfigurableReflectedAction<TestPlugin.TestOptions>) action;
			Assert.That(configurable.Options, Has.Count.EqualTo(2), "Action options weren't maintained properly");
		}
	}
	private static PluginAction AnalyzeActionInfo(IEnumerator<PluginAction> enumerator, string name, string? description = null) {
		Console.WriteLine($"Testing action '{name}'");

		Assert.That(enumerator.MoveNext(), Is.True, $"Action '{name}' wasn't returned by reflector");
		var action = enumerator.Current;
		
		AssertInfo(action.Name, action.Description, name, description, "Action");

		return action;
	}
	
	private static void AssertInfo(string actualName, string? actualDescription, string name, string? description, string logPrefix) {
		Assert.That(actualName, Is.EqualTo(name), $"{logPrefix} '{name}' had invalid title");

		if (description != null)
			Assert.That(actualDescription, Is.EqualTo(description), $"{logPrefix} '{name}' had invalid description");
	}
}