using System;
using System.Collections.Generic;
using System.Diagnostics.CodeAnalysis;
using System.Linq;
using System.Threading.Tasks;
using Streamduck.Attributes;
using Streamduck.Plugins;
using Streamduck.Scripting;
using Desc = Streamduck.Attributes.DescriptionAttribute;

namespace StreamduckTest; 

[TestFixture]
[SuppressMessage("Assertion", "NUnit2045:Use Assert.Multiple")]
public class PluginReflectorTest {
	private class TestPlugin : Plugin {
		public override string Name => "Test Plugin";

		public bool ActionWasCalled { get; private set; }

		[PluginMethod]
		public async Task TestAsyncAction(int value) {
			if (value > 5) {
				await Task.Delay(3);
				ActionWasCalled = true;
			}
		}

		[PluginMethod]
		public async Task<int> TestAsyncFunction(int value) {
			await Task.Delay(3);
			return value * value;
		}
		
		[PluginMethod]
		[Name("Renamed Action")]
		[Desc("Action description")]
		public void TestAction([Name("Renamed Value")] [Desc("With description")] int value) {
			if (value > 5) {
				ActionWasCalled = true;
			}
		}

		[PluginMethod]
		[Name("Square Value")]
		[Desc("This function squares numbers")]
		[return: Name("Square Output")]
		[return: Desc("Squared output of inputted value")]
		public int Square(
			[Name("Square Input")]
			[Desc("Number to square")]
			int value
		) => value * value;
	}

	[Test]
	public void TestActions() {
		var plugin = new TestPlugin();

		var methods = PluginReflector.GetMethods(plugin);
		using var actions = PluginReflector.AnalyzeActions(methods, plugin).GetEnumerator();

		{ // Test Action
			var action = AnalyzeActionInfo(actions, "Renamed Action", "Action description");

			using var parameters = action.Parameters.AsEnumerable().GetEnumerator();
			AnalyzeDataInfo<int>(parameters, "Renamed Value", "With description");
			
			action.Invoke(new object[]{ 6 });
			Assert.That(plugin.ActionWasCalled, Is.True, "Action was not properly called");

			Assert.Catch<ArgumentException>(() => action.Invoke(new object[] { 6.0 }),
				"Action doesn't throw exception when arguments are invalid");
		}
	}

	[Test]
	public void TestFunctions() {
		var plugin = new TestPlugin();

		var methods = PluginReflector.GetMethods(plugin);
		using var functions = PluginReflector.AnalyzeFunctions(methods, plugin).GetEnumerator();

		{ // Square function test
			var function = AnalyzeFunctionInfo(functions, "Square Value", "This function squares numbers");

			using var parameters = function.Parameters.AsEnumerable().GetEnumerator();
			AnalyzeDataInfo<int>(parameters, "Square Input", "Number to square");

			using var returns = function.Returns.AsEnumerable().GetEnumerator();
			AnalyzeDataInfo<int>(returns, "Square Output", "Squared output of inputted value");

			var output = function.Invoke(new object[] { 5 });
			Assert.That(output, Is.Not.Empty, "Square function didn't return anything");

			var value = output[0];
			Assert.That(value, Is.Not.Null, "Square function returned null");
			Assert.That(value, Is.InstanceOf<int>(), "Square function returned something else than integer");
			
			Assert.That((int) value!, Is.EqualTo(25), "Square function calculation was incorrect");
		}
	}

	[Test]
	public async Task TestAsyncActions() {
		var plugin = new TestPlugin();
		
		var methods = PluginReflector.GetMethods(plugin);
		using var actions = PluginReflector.AnalyzeAsyncActions(methods, plugin).GetEnumerator();
		
		{ // Test Action
			var action = AnalyzeAsyncActionInfo(actions, "Test Async Action");

			using var parameters = action.Parameters.AsEnumerable().GetEnumerator();
			AnalyzeDataInfo<int>(parameters, "Value");
			
			await action.Invoke(new object[]{ 6 });
			Assert.That(plugin.ActionWasCalled, Is.True, "Action was not properly called");

			Assert.CatchAsync<ArgumentException>(async () => await action.Invoke(new object[] { 6.0 }),
				"Action doesn't throw exception when arguments are invalid");
		}
	}
	
	[Test]
	public async Task TestAsyncFunctions() {
		var plugin = new TestPlugin();
		
		var methods = PluginReflector.GetMethods(plugin);
		using var functions = PluginReflector.AnalyzeAsyncFunctions(methods, plugin).GetEnumerator();
		
		{ // Test Action
			var function = AnalyzeAsyncFunctionInfo(functions, "Test Async Function");

			using var parameters = function.Parameters.AsEnumerable().GetEnumerator();
			AnalyzeDataInfo<int>(parameters, "Value");
			
			using var returns = function.Returns.AsEnumerable().GetEnumerator();
			AnalyzeDataInfo<int>(returns, "Out");
			
			var output = await function.Invoke(new object[]{ 6 });
			Assert.That(output, Is.Not.Empty, "Square function didn't return anything");
			
			var value = output[0];
			Assert.That(value, Is.InstanceOf<int>(), "Square function returned something else than integer");
			
			Assert.That((int) value, Is.EqualTo(36), "Square function calculation was incorrect");
		}
	}

	private static PluginAction AnalyzeActionInfo(IEnumerator<PluginAction> enumerator, string name, string? description = null) {
		Console.WriteLine($"Testing action '{name}'");

		Assert.That(enumerator.MoveNext(), Is.True, $"Action '{name}' wasn't returned by reflector");
		var action = enumerator.Current;
		
		AssertInfo(action.Name, action.Description, name, description, "Action");

		return action;
	}

	private static PluginFunction AnalyzeFunctionInfo(IEnumerator<PluginFunction> enumerator, string name, string? description = null) {
		Console.WriteLine($"Testing function '{name}'");
		
		Assert.That(enumerator.MoveNext(), Is.True, $"Function '{name}' wasn't returned by reflector");
		var function = enumerator.Current;

		AssertInfo(function.Name, function.Description, name, description, "Function");

		return function;
	}
	
	private static AsyncPluginAction AnalyzeAsyncActionInfo(IEnumerator<AsyncPluginAction> enumerator, string name, string? description = null) {
		Console.WriteLine($"Testing async action '{name}'");

		Assert.That(enumerator.MoveNext(), Is.True, $"Async action '{name}' wasn't returned by reflector");
		var action = enumerator.Current;
		
		AssertInfo(action.Name, action.Description, name, description, "Async action");

		return action;
	}

	private static AsyncPluginFunction AnalyzeAsyncFunctionInfo(IEnumerator<AsyncPluginFunction> enumerator, string name, string? description = null) {
		Console.WriteLine($"Testing async function '{name}'");
		
		Assert.That(enumerator.MoveNext(), Is.True, $"Async function '{name}' wasn't returned by reflector");
		var function = enumerator.Current;

		AssertInfo(function.Name, function.Description, name, description, "Async function");

		return function;
	}
	
	private static void AssertInfo(string actualName, string? actualDescription, string name, string? description, string logPrefix) {
		Assert.That(actualName, Is.EqualTo(name), $"{logPrefix} '{name}' had invalid title");

		if (description != null)
			Assert.That(actualDescription, Is.EqualTo(description), $"{logPrefix} '{name}' had invalid description");
	}

	private static void AnalyzeDataInfo<T>(IEnumerator<DataInfo> enumerator, string name, string? description = null) {
		Console.WriteLine($"Testing parameter '{name}'");
		
		Assert.That(enumerator.MoveNext(), Is.True, $"Parameter '{name}' wasn't returned by reflector");
		var info = enumerator.Current;
		
		Assert.That(info.Type, Is.EqualTo(typeof(T)), $"Parameter '{name}' had invalid type");
		Assert.That(info.Name, Is.EqualTo(name), $"Parameter '{name}' had invalid title");
		
		if (description != null) 
			Assert.That(info.Description, Is.EqualTo(description), $"Parameter '{name}' had invalid description");
	}
}