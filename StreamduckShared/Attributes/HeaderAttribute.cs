using System;

namespace Streamduck.Attributes; 

/**
 * Display a header before the property
 */
[AttributeUsage(AttributeTargets.Property)]
public class HeaderAttribute(string text) : Attribute {
	public string Text { get; } = text;
}