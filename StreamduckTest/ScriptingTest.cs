using System;
using System.Diagnostics.CodeAnalysis;
using System.Text;
using System.Text.Json;
using System.Threading.Tasks;
using Avalonia.Controls;
using Streamduck.Attributes;
using Streamduck.Inputs;
using Streamduck.Plugins;
using Streamduck.Scripting;
using Streamduck.Utils;

namespace StreamduckTest; 

[TestFixture]
[SuppressMessage("Assertion", "NUnit2045:Use Assert.Multiple")]
public class ScriptingTest {
	public class TestActionPlugin : Plugin {
		public override string Name => "Test Action Plugin";

		public bool ActionCalled { get; private set; }
		
		[PluginMethod]
		public void TestAction() {
			ActionCalled = true;
		}
	}

	public class TestScriptingSystem : ScriptingSystem {
		public class TestScript : Script {
			private readonly IPluginQuery _pluginQuery;
			
			public TestScript(IPluginQuery query) {
				_pluginQuery = query;
			}

			public override ScriptInstance MakeInstance() => new TestScriptInstance(this);

			public override Task<byte[]> Serialize() {
				var json = JsonSerializer.Serialize(ActionName);
				var bytes = Encoding.UTF8.GetBytes(json);
				return Task.FromResult(bytes);
			}
			
			public NamespacedName? ActionName { get; set; }

			public void Execute() {
				if (ActionName == null) return;
				var action = _pluginQuery.SpecificPluginAction(ActionName.Value);

				action?.Instance.Invoke(Array.Empty<object>());
			}
		}

		public class TestScriptInstance : ScriptInstance {
			public TestScriptInstance(Script _script) : base(_script) {
				pressed = () => {
					if (this._script.WeakToNullable() is not TestScript script) return;
					script.Execute();
				};
			}

			private readonly Action pressed;
			
			public override void AddHooks(Input input) {
				if (input is IInputButton button) {
					button.ButtonPressed += pressed;
				}
			}

			public override void RemoveHooks(Input input) {
				if (input is IInputButton button) {
					button.ButtonPressed -= pressed;
				}
			}
		}
		
		public IPluginQuery? Query { get; init; }

		public override string Name => "Test Scripting System";
		public override Task<Script> New() => Task.FromResult<Script>(new TestScript(Query!));
		
		public override Control Editor(Script script) => throw new NotImplementedException();

		public override Task<Script> Deserialize(byte[] data) {
			var json = Encoding.UTF8.GetString(data);
			var name = JsonSerializer.Deserialize<NamespacedName>(json);
			return Task.FromResult<Script>(new TestScript(Query!) { ActionName = name });
		}
	}

	public class TestInput : Input, IInputButton {
		public TestInput() : base(0, 0, 1, 1, InputIcon.Button) { }
		
		public event Action? ButtonPressed;
		public event Action? ButtonReleased;

		public void Press() {
			ButtonPressed?.Invoke();
		}
		
		public void Release() {
			ButtonReleased?.Invoke();
		}
	}

	[Test]
	public async Task TestScriptCallingAction() {
		var actionPlugin = new TestActionPlugin();
		var collection = new PluginCollection(new[] { actionPlugin });
		var input = new TestInput();

		var scriptingSystem = new TestScriptingSystem { Query = collection };

		var script = (await scriptingSystem.New() as TestScriptingSystem.TestScript)!;
		script.ActionName = new NamespacedName("Test Action Plugin", "Test Action");

		var scriptInstance = script.MakeInstance();
		scriptInstance.AddHooks(input);
		
		input.Press();
		
		Assert.That(actionPlugin.ActionCalled, Is.True, "Script didn't do anything");
	}
	
	[Test]
	public async Task TestScriptSerialization() {
		var collection = new PluginCollection(Array.Empty<Plugin>());

		var scriptingSystem = new TestScriptingSystem { Query = collection };

		var script = (await scriptingSystem.New() as TestScriptingSystem.TestScript)!;
		script.ActionName = new NamespacedName("Test Action Plugin", "Test Action");

		// Serialize
		var data = await script.Serialize();

		var deserializedScript = await scriptingSystem.Deserialize(data);
		Assert.That(deserializedScript, Is.InstanceOf<TestScriptingSystem.TestScript>(), 
			"Scripting system provided incorrect instance");
		
		Assert.That(((TestScriptingSystem.TestScript) deserializedScript).ActionName, Is.EqualTo(script.ActionName), 
			"Action name wasn't correct after deserialization");
	}
}