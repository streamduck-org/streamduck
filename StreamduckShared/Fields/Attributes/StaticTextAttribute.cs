using System;

namespace Streamduck.Fields.Attributes;

[AttributeUsage(AttributeTargets.Property, AllowMultiple = true)]
public class StaticTextAttribute : Attribute {
	public StaticTextAttribute(string text) {
		Text = text;
	}
	
	public string Text { get; }
}