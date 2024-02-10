// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

using System;

namespace Streamduck.Attributes;

/**
 * Specifies description for the property, or description for Action/Function parameters or returns
 */
[AttributeUsage(AttributeTargets.Property | AttributeTargets.Field | AttributeTargets.Method
                | AttributeTargets.Parameter | AttributeTargets.ReturnValue)]
public class DescriptionAttribute(string description) : Attribute {
	public string Description { get; } = description;
}