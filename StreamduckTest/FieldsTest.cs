using System;
using System.Collections.Generic;
using System.Linq;
using Streamduck.Fields;
using Streamduck.Fields.Attributes;

namespace StreamduckTest; 

[TestFixture]
public class FieldsTest {
	private class TestOptions {
		public string LabelTest => "test";
		
		[ReadOnly]
		public string AlsoLabel { get; set; } = "test";
	}

	[Test]
	public void TestAnalyzedFields() {
		using var fields = FieldReflector.AnalyzeObject(new TestOptions()).GetEnumerator();
		
		AssertLabel(fields, "Label Test", "test");
		AssertLabel(fields, "Also Label", "test");
	}

	private static void AssertLabel(IEnumerator<Field> enumerator, string title, string text) {
		Console.WriteLine($"Testing field '{title}'");
		
		Assert.That(enumerator.MoveNext(), Is.True, $"Field '{title}' wasn't returned by reflector");
		var field = enumerator.Current;
		
		Assert.That(field, Is.Not.Null, $"Field '{title}' was null");
		Assert.That(field, Is.InstanceOf<Field.Label>(), $"Field '{title}' wasn't a label");
		Assert.That(field!.Title, Is.EqualTo(title), $"Field '{title}' had invalid title");

		var labelField = field as Field.Label;
		Assert.That(labelField!.TextAccessor.Invoke(), Is.EqualTo(text), $"Field '{title}' returned invalid text");
	}
}