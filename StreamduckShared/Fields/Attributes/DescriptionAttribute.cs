using System;

namespace Streamduck.Fields.Attributes;

/**
 * Specifies description for the property
 */
[AttributeUsage(AttributeTargets.Property)]
public class DescriptionAttribute : Attribute {
	public DescriptionAttribute(string description) {
		Description = description;
	}
	
	public string Description { get; }
}