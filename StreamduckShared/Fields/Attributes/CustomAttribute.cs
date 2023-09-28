using System;

namespace Streamduck.Fields.Attributes; 

/**
 * Will display field using the custom Avalonia Control type,
 * constructor with type of the property is expected as the only parameter.
 * Property title and description if provided will be shown above the control.
 * <remarks>
 * Use [Name("")] to remove title.
 * </remarks>
 */
[AttributeUsage(AttributeTargets.Property)]
public class CustomAttribute : Attribute {
	public CustomAttribute(Type userControlType) {
		UserControlType = userControlType;
	}

	public Type UserControlType { get; }
}