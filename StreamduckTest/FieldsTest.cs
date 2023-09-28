using System;
using System.Collections.Generic;
using System.Linq;
using System.Numerics;
using Streamduck.Fields;
using Streamduck.Fields.Attributes;

namespace StreamduckTest; 

[TestFixture]
public class FieldsTest {
	private class TestOptions {
		public string LabelTest => "test";
		
		[ReadOnly]
		public string ReadonlyStringInput { get; set; } = "test";

		[Header("Header test")]
		
		[StaticText("Here's some static text")]
		[StaticText("And another line that should usually go sequentially")]
		
		public string TextField { get; set; } = "what";

		[NumberField<int>(1, 2, false)]
		public int Integer { get; set; } = 1;
		public int DefaultInteger { get; set; } = 1;

		[Name("Something else")]
		[NumberField<double>(0, 1000, enforceLimit: true)]
		public double FloatingNumber { get; private set; } = 420.69;
	}

	[Test]
	public void TestAnalyzedFields() {
		using var fields = FieldReflector.AnalyzeObject(new TestOptions()).GetEnumerator();
		
		AssertLabel(fields, "Label Test", "test");
		AssertStringInput(fields, "Readonly String Input", "test", true);
		
		AssertHeader(fields, "Header test");
		AssertStaticText(fields, "Here's some static text");
		AssertStaticText(fields, "And another line that should usually go sequentially");
		
		AssertStringInput(fields, "Text Field", "what", false);
		
		AssertNumberField(fields, "Integer", 1, false, false, false, 1, 2);
		AssertNumberField(fields, "Default Integer", 1, false, true, false, 0, 1);
		AssertNumberField(fields, "Something else", 420.69, true, true, true, 0, 1000);
	}

	private static void AssertLabel(IEnumerator<Field> enumerator, string title, string text) {
		Console.WriteLine($"Testing field '{title}'");
		
		Assert.That(enumerator.MoveNext(), Is.True, $"Field '{title}' wasn't returned by reflector");
		var field = enumerator.Current;
		
		Assert.That(field, Is.Not.Null, $"Field '{title}' was null");
		Assert.That(field, Is.InstanceOf<Field.Label>(), $"Field '{title}' wasn't a label");
		Assert.That(field.Title, Is.EqualTo(title), $"Field '{title}' had invalid title");

		var labelField = field as Field.Label;
		Assert.That(labelField!.Text, Is.EqualTo(text), $"Field '{title}' returned invalid text");
	}
	
	private static void AssertHeader(IEnumerator<Field> enumerator, string title) {
		Console.WriteLine($"Testing field '{title}'");
		
		Assert.That(enumerator.MoveNext(), Is.True, $"Field '{title}' wasn't returned by reflector");
		var field = enumerator.Current;
		
		Assert.That(field, Is.Not.Null, $"Field '{title}' was null");
		Assert.That(field, Is.InstanceOf<Field.Header>(), $"Field '{title}' wasn't a header");
		Assert.That(field.Title, Is.EqualTo(title), $"Field '{title}' had invalid text");
	}
	
	private static void AssertStaticText(IEnumerator<Field> enumerator, string title) {
		Console.WriteLine($"Testing field '{title}'");
		
		Assert.That(enumerator.MoveNext(), Is.True, $"Field '{title}' wasn't returned by reflector");
		var field = enumerator.Current;
		
		Assert.That(field, Is.Not.Null, $"Field '{title}' was null");
		Assert.That(field, Is.InstanceOf<Field.StaticText>(), $"Field '{title}' wasn't a static text");
		Assert.That(field.Title, Is.EqualTo(title), $"Field '{title}' had invalid text");
	}
	
	private static void AssertStringInput(IEnumerator<Field> enumerator, string title, string text, bool readOnly) {
		Console.WriteLine($"Testing field '{title}'");
		
		Assert.That(enumerator.MoveNext(), Is.True, $"Field '{title}' wasn't returned by reflector");
		var field = enumerator.Current;
		
		Assert.That(field, Is.Not.Null, $"Field '{title}' was null");
		Assert.That(field, Is.InstanceOf<Field.StringInput>(), $"Field '{title}' wasn't a string input");
		Assert.That(field.Title, Is.EqualTo(title), $"Field '{title}' had invalid title");

		var stringInput = field as Field.StringInput;
		Assert.That(stringInput!.Value, Is.EqualTo(text), $"Field '{title}' returned invalid text");
		
		var newText = stringInput.Value + "new test value";
		stringInput.Value = newText;

		if (readOnly) Assert.That(stringInput.Value, Is.EqualTo(text), $"Field '{title}' updated its value despite being read-only");
		else Assert.That(stringInput.Value, Is.EqualTo(newText), $"Field '{title}' didn't update its value after being set");
	}
	
	private static void AssertNumberField<T>(IEnumerator<Field> enumerator, string title, T value, 
		bool readOnly, bool slider, bool enforce, T min, T max) where T : INumber<T> {
		Console.WriteLine($"Testing field '{title}'");
		
		Assert.That(enumerator.MoveNext(), Is.True, $"Field '{title}' wasn't returned by reflector");
		var field = enumerator.Current;
		
		Assert.That(field, Is.Not.Null, $"Field '{title}' was null");
		Assert.That(field, Is.InstanceOf<Field.NumberInput<T>>(), $"Field '{title}' wasn't a correct number input");
		Assert.That(field.Title, Is.EqualTo(title), $"Field '{title}' had invalid title");

		var numberInput = field as Field.NumberInput<T>;
		Assert.That(numberInput!.Value, Is.EqualTo(value), $"Field '{title}' didn't have a correct value");

		var newValue = numberInput.Value + T.One;
		numberInput.Value = newValue;
		
		if (readOnly) Assert.That(numberInput.Value, Is.EqualTo(value), $"Field '{title}' updated its value despite being read-only");
		else Assert.That(numberInput.Value, Is.EqualTo(newValue), $"Field '{title}' didn't update its value after being set");
		
		Assert.That(numberInput.Slider, Is.EqualTo(slider), $"Field '{title}' had invalid slider setting");
		Assert.That(numberInput.EnforceLimit, Is.EqualTo(enforce), $"Field '{title}' had invalid enforce limit setting");
		Assert.That(numberInput.Min, Is.EqualTo(min), $"Field '{title}' had invalid min setting");
		Assert.That(numberInput.Max, Is.EqualTo(max), $"Field '{title}' had invalid max setting");
	}
}