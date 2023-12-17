using System;

namespace Streamduck.Attributes;

[AttributeUsage(AttributeTargets.Property, AllowMultiple = true)]
public class StaticTextAttribute(string text) : Attribute {
	public string Text { get; } = text;
}