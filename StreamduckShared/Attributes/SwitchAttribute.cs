using System;

namespace Streamduck.Attributes; 

/**
 * Makes boolean properties get switch style of a toggle
 */
[AttributeUsage(AttributeTargets.Property)]
public class SwitchAttribute : Attribute { }