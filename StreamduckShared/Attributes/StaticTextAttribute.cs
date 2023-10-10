using System;

namespace Streamduck.Attributes;

[AttributeUsage(AttributeTargets.Property, AllowMultiple = true)]
public class StaticTextAttribute : Attribute {
	public StaticTextAttribute(string text) {
		Text = text;
	}
	
	public string Text { get; }
}