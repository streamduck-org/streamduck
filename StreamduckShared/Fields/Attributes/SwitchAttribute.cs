using System;

namespace Streamduck.Fields.Attributes; 

/**
 * Makes boolean properties get switch style of a toggle
 */
[AttributeUsage(AttributeTargets.Property)]
public class SwitchAttribute : Attribute { }