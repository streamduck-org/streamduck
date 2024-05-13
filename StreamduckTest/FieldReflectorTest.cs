// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

using System;
using System.Collections.Generic;
using System.Diagnostics.CodeAnalysis;
using System.Linq;
using System.Numerics;
using Streamduck.Attributes;
using Streamduck.Fields;
using Desc = Streamduck.Attributes.DescriptionAttribute;

namespace StreamduckTest;

[TestFixture]
[SuppressMessage("Assertion", "NUnit2045:Use Assert.Multiple")]
public class FieldReflectorTest {
	private class TestOptions {
		[Desc("Description on label")] public string LabelTest => "test";

		[ReadOnly]
		[Desc("Description on string input")]
		public string ReadonlyStringInput { get; set; } = "test";

		[Header("Header test")]
		[StaticText("Here's some static text")]
		[StaticText("And another line that should usually go sequentially")]

		public string TextField { get; set; } = "what";

		[NumberField<int>(1, 2, false)] public int Integer { get; set; } = 1;

		[Include(true)]
		[Desc("Description on number")]
		private int DefaultInteger { get; set; } = 1;

		[Name("Something else")]
		[NumberField<double>(0, 1000, enforceLimit: true)]
		public double FloatingNumber { get; private set; } = 420.69;

		[Desc("Description on boolean")] public bool Checkbox { get; set; }

		[Switch] public bool ReadonlySwitch { get; private set; } = true;

		[Desc("Description on enum")] public TestEnum DropdownValue { get; set; } = TestEnum.FirstVariant;

		[ReadOnly] public TestEnum ReadonlyDropdownValue { get; set; } = TestEnum.SecondVariant;

		[Bitmask]
		[Desc("Description on bitmask")]
		public TestBitmask BitmaskValue { get; set; } = TestBitmask.One;

		[Desc("Description on nested field")] public NestedOptions MoreOptions { get; } = new();
	}

	private enum TestEnum {
		[Name("Renamed Variant")] FirstVariant,
		[Desc("Description on enum variant")] SecondVariant,
		ThirdVariant,
		ForthVariant = 4
	}

	[Flags]
	private enum TestBitmask {
		One = 1,
		[Desc("This is two")] Two = 2,
		OneAndTwo = 3
	}

	private class NestedOptions {
		public int MoreOptionsHere { get; set; }
		public float OfDifferentTypes { get; set; }
		public string EvenGotStrings => "";
	}

	[Test]
	public void TestAnalyzedFields() {
		using var fields = FieldReflector.AnalyzeObject(new TestOptions()).GetEnumerator();

		AssertLabel(fields, "Label Test", "test", "Description on label");
		AssertStringInput(fields, "Readonly String Input", "test", true, "Description on string input");

		AssertHeader(fields, "Header test");
		AssertStaticText(fields, "Here's some static text");
		AssertStaticText(fields, "And another line that should usually go sequentially");

		AssertStringInput(fields, "Text Field", "what", false);

		AssertNumberField(fields, "Integer", 1, false, false, false, 1, 2);
		AssertNumberField(fields, "Default Integer", 1, false, true, false, 0, 1, "Description on number");
		AssertNumberField(fields, "Something else", 420.69, true, true, true, 0, 1000);

		AssertCheckbox(fields, "Checkbox", false, false, false, "Description on boolean");
		AssertCheckbox(fields, "Readonly Switch", true, true, true);

		AssertEnum(fields, "Dropdown Value", "Renamed Variant", "Third Variant", false);
		AssertEnum(fields, "Readonly Dropdown Value", "Second Variant", "Third Variant", true);

		{
			// Testing bitmask
			const string title = "Bitmask Value";
			var bitmaskField = AssertFieldInfo<Field.MultiChoice>(fields, title, "Description on bitmask");
			Assert.That(bitmaskField.Variants, Is.Not.Empty, $"Field '{title}' doesn't have any variants");
			Console.WriteLine($"Field has following variants: {string.Join(", ", bitmaskField.Variants)}");

			Assert.That(
				bitmaskField.Variants[1].Item2, Is.EqualTo("This is two"),
				$"Field '{title}' 2nd variant had invalid description"
			);

			bitmaskField["One And Two"] = true;
			Assert.That(bitmaskField["Two"], Is.True, $"Field '{title}' incorrectly handled bitmasks");

			bitmaskField["One"] = false;
			Assert.That(bitmaskField["One And Two"], Is.False, $"Field '{title}' incorrectly handled bitmasks");
		}

		{
			// Testing nested objects
			const string title = "More Options";
			var nestedField = AssertFieldInfo<Field.NestedFields>(fields, title, "Description on nested field");

			var nestedFields = nestedField.Schema.AsEnumerable().GetEnumerator();
			AssertNumberField(nestedFields, "More Options Here", 0, false, true, false, 0, 1);
			AssertNumberField(nestedFields, "Of Different Types", 0.0f, false, true, false, 0.0f, 1.0f);
			AssertLabel(nestedFields, "Even Got Strings", "");
		}
	}

	private static T AssertFieldInfo<T>(IEnumerator<Field> enumerator, string title, string? description = null)
		where T : class {
		Console.WriteLine($"Testing field '{title}'");

		Assert.That(enumerator.MoveNext(), Is.True, $"Field '{title}' wasn't returned by reflector");
		var field = enumerator.Current;

		Assert.That(field, Is.Not.Null, $"Field '{title}' was null");
		Assert.That(field, Is.InstanceOf<T>(), $"Field '{title}' wasn't of valid type");
		Assert.That(field.Title, Is.EqualTo(title), $"Field '{title}' had invalid title");

		if (description != null)
			Assert.That(field.Description, Is.EqualTo(description), $"Field '{title}' had invalid description");

		return (field as T)!;
	}

	private static void AssertLabel(IEnumerator<Field> enumerator, string title, string text,
		string? description = null
	) {
		var labelField = AssertFieldInfo<Field.Label>(enumerator, title, description);
		Assert.That(labelField!.Text, Is.EqualTo(text), $"Field '{title}' returned invalid text");
	}

	private static void AssertHeader(IEnumerator<Field> enumerator, string title) {
		AssertFieldInfo<Field.Header>(enumerator, title);
	}

	private static void AssertStaticText(IEnumerator<Field> enumerator, string title) {
		AssertFieldInfo<Field.StaticText>(enumerator, title);
	}

	private static void AssertStringInput(IEnumerator<Field> enumerator, string title, string text, bool readOnly,
		string? description = null
	) {
		var stringInput = AssertFieldInfo<Field.StringInput>(enumerator, title, description);
		Assert.That(stringInput!.Value, Is.EqualTo(text), $"Field '{title}' returned invalid text");

		var newText = stringInput.Value + "new test value";
		stringInput.Value = newText;

		if (readOnly)
			Assert.That(
				stringInput.Value, Is.EqualTo(text),
				$"Field '{title}' updated its value despite being read-only"
			);
		else
			Assert.That(
				stringInput.Value, Is.EqualTo(newText),
				$"Field '{title}' didn't update its value after being set"
			);
	}

	private static void AssertNumberField<T>(IEnumerator<Field> enumerator, string title, T value,
		bool readOnly, bool slider, bool enforce, T min, T max, string? description = null
	) where T : INumber<T> {
		var numberInput = AssertFieldInfo<Field.NumberInput<T>>(enumerator, title, description);
		Assert.That(numberInput!.Value, Is.EqualTo(value), $"Field '{title}' didn't have a correct value");

		var newValue = numberInput.Value + T.One;
		numberInput.Value = newValue;

		if (readOnly)
			Assert.That(
				numberInput.Value, Is.EqualTo(value),
				$"Field '{title}' updated its value despite being read-only"
			);
		else
			Assert.That(
				numberInput.Value, Is.EqualTo(newValue),
				$"Field '{title}' didn't update its value after being set"
			);

		Assert.That(numberInput.Slider, Is.EqualTo(slider), $"Field '{title}' had invalid slider setting");
		Assert.That(
			numberInput.EnforceLimit, Is.EqualTo(enforce),
			$"Field '{title}' had invalid enforce limit setting"
		);
		Assert.That(numberInput.Min, Is.EqualTo(min), $"Field '{title}' had invalid min setting");
		Assert.That(numberInput.Max, Is.EqualTo(max), $"Field '{title}' had invalid max setting");
	}

	private static void AssertCheckbox(IEnumerator<Field> enumerator, string title, bool value, bool readOnly,
		bool switchStyle, string? description = null
	) {
		var checkbox = AssertFieldInfo<Field.Checkbox>(enumerator, title, description);
		Assert.That(checkbox!.Value, Is.EqualTo(value), $"Field '{title}' returned invalid state");

		var newValue = !checkbox.Value;
		checkbox.Value = newValue;

		Assert.That(checkbox.SwitchStyle, Is.EqualTo(switchStyle), $"Field '{title}' has invalid style");

		if (readOnly)
			Assert.That(
				checkbox.Value, Is.EqualTo(value),
				$"Field '{title}' updated its value despite being read-only"
			);
		else
			Assert.That(
				checkbox.Value, Is.EqualTo(newValue),
				$"Field '{title}' didn't update its value after being set"
			);
	}

	private static void AssertEnum(IEnumerator<Field> enumerator, string title, string currentValue, string newValue,
		bool readOnly, string? description = null, string? valueDescription = null
	) {
		var enumField = AssertFieldInfo<Field.Choice>(enumerator, title, description);
		Assert.That(enumField!.Value, Is.EqualTo(currentValue), $"Field '{title}' returned invalid value");
		Assert.That(enumField.Variants, Is.Not.Empty, $"Field '{title}' didn't have any variants listed");
		Console.WriteLine($"Field has following variants: {string.Join(", ", enumField.Variants)}");
		enumField.Value = newValue;

		if (readOnly)
			Assert.That(
				enumField.Value, Is.EqualTo(currentValue),
				$"Field '{title}' updated its value despite being read-only"
			);
		else
			Assert.That(
				enumField.Value, Is.EqualTo(newValue),
				$"Field '{title}' didn't update its value after being set"
			);
	}
}