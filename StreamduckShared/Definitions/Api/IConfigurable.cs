using System.Numerics;
using System.Reflection;
using System.Threading.Tasks;

namespace Streamduck.Definitions.Api; 

/* TODO: Put this information somewhere in Field documentation
 * Options objects will be queried using reflection for public properties and/or attributed properties.
 * - Property that has getter and setter can be edited in UI
 * - Property that only has getter will no be able to be edited in UI and will be shown as read-only
 * - Non-public property annotated with any derivitive of [Field] will be shown in UI and read-only by default (can be read-write if specified in the attribute)
 * - Non-public properties not annotated with any of [Field] attributes will not be shown in UI
 * 
 * Property field type will be assumed from it's Type, use any of the [Field] attributes to force field type 
*/

/**
 * Object provided as Options needs to support serialization,
 * along with having fields that can be represented as Field objects
 */
public interface IConfigurable {
	object Options { get; }
}

/**
 * Type provided as Options needs to support serialization and deserialization,
 * along with having class fields that can be represented as Field objects.
 */
public interface IConfigurable<T> : IConfigurable {
	object IConfigurable.Options => Options!;
	new T Options { get; set; }
}