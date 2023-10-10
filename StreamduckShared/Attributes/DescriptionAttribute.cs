using System;

namespace Streamduck.Attributes;

/**
 * Specifies description for the property, or description for Action/Function parameters or returns
 */
[AttributeUsage(AttributeTargets.Property | AttributeTargets.Field | AttributeTargets.Method 
                | AttributeTargets.Parameter | AttributeTargets.ReturnValue)]
public class DescriptionAttribute : Attribute {
	public DescriptionAttribute(string description) {
		Description = description;
	}
	
	public string Description { get; }
}