// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

using System;

namespace Streamduck.Attributes;

/**
 * Will display field using the custom Avalonia Control type,
 * constructor with type of the property is expected as the only parameter.
 * Property title and description if provided will be shown above the control.
 * <remarks>
 *     Use [Name("")] to remove title.
 * </remarks>
 */
[AttributeUsage(AttributeTargets.Property)]
public class CustomAttribute(Type userControlType) : Attribute {
	public Type UserControlType { get; } = userControlType;
}