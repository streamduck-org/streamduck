using System;

namespace Streamduck.Attributes; 

/**
 * Display a header before the property
 */
[AttributeUsage(AttributeTargets.Property)]
public class HeaderAttribute : Attribute {
	public HeaderAttribute(string text) {
		Text = text;
	}
	
	public string Text { get; }
}