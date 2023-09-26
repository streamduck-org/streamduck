using System;
using System.Linq;
using Streamduck.Fields;
using Streamduck.Fields.Attributes;

namespace StreamduckTest; 

[TestFixture]
public class FieldsTest {
	private class TestOptions {
		public string LabelTest => "test";
	}

	[Test]
	public void TestForExitence() {
		var fields = FieldReflector.AnalyzeObject(new TestOptions()).ToArray();
		Assert.That(fields, Is.Not.Empty);

		var label = fields.FirstOrDefault();
		Assert.That(label, Is.Not.Null);
		Assert.That(label, Is.InstanceOf<Field.Label>());
		Assert.That(label!.Title, Is.EqualTo("Label Test"));

		var labelField = label as Field.Label;
		Assert.That(labelField!.TextAccessor.Invoke(), Is.EqualTo("test"));
	}
}