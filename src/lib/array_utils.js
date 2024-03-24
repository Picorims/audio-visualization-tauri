//Wav2Bar - Free software for creating audio visualization (motion design) videos
//Copyright (C) 2023  Picorims <picorims.contact@gmail.com>

//This program is free software: you can redistribute it and/or modify
//it under the terms of the GNU General Public License as published by
//the Free Software Foundation, either version 3 of the License, or
//any later version.

//This program is distributed in the hope that it will be useful,
//but WITHOUT ANY WARRANTY; without even the implied warranty of
//MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
//GNU General Public License for more details.

//You should have received a copy of the GNU General Public License
//along with this program.  If not, see <https://www.gnu.org/licenses/>.

import * as Type from "./type_checking.js";

//array related utilities

/**
 * function that remaps an array, within the given min and max, to a new length.
 *
 * @export
 * @param {Array} array
 * @param {Number} new_length
 * @param {Number} min minimum index to consider for mapping
 * @param {Number} max maximum index to consider for mapping
 * @return {Array} The mapped array.
 */
export function MappedArray(array, new_length, min, max) {

    //CHECK VARIABLES
    if ( !Type.IsAnArray(array) )                    throw `MappedArray: ${array} is not an array!`;
    if ( !Type.IsANumber(new_length) )               throw `MappedArray: ${new_length} is not a number!`;
    if ( !Type.IsUndefined(min) && !Type.IsANumber(min) ) throw `MappedArray: ${min} is not a number!`; //min and max are optional and undefined shouldn't
    if ( !Type.IsUndefined(max) && !Type.IsANumber(max) ) throw `MappedArray: ${max} is not a number!`; //trigger any error.
    for (let i = 0; i < array.length; i++) {
        if ( Type.IsUndefined(array[i]) ) throw `MappedArray: the value ${i} of the array is undefined or null!`;
    }

    //DEFINITIONS
    if ( Type.IsUndefined(min) || Type.IsUndefined(max) ) {//if min or max not specified.
        min = 0;
        max = array.length-1;
    }

    var new_array = [];
    var step = (   (max-min+1) / new_length   ) * new_length / (new_length-1);//range length / new length.
    //Proportionality to one less equal part (* new_length / (new_length-1)) is here so the step goes up to the last
    //value of the array when dividing the range into equal parts. (as the final increment would otherwise stop 1 equal part before the last value).

    var increment = min;//we start a the minimum of the range

    //We want to take at equal distance a "new_length" number of values in the old array, from min to max.
    //In order to know how much we need to increment, we create a step.
    //If the range length is inferior than the new length, step < 1 since we have to get some values multiple times
    //to match the new length.
    //If the range length is superior than the new length, step > 1 since we have to skip some values to match the new length.




    //ARRAY CREATION
    for (let i = 0; i<new_length; i++) {
        if (increment === array.length) {     new_array.push(  array[ parseInt(increment-1) ]  );     }
        else                            {     new_array.push(  array[ parseInt(increment) ]    );     }
        increment += step;
    }

    //RETURN THE NEW ARRAY TO THE CALL
    return new_array;
}

/**
 * redistributes the indexes in a logarithmic base 10 scale
 *
 * @export
 * @param {Array} array source array
 * @return {Array} logarithmic array
 */
export function LinearToLog(array) {
    if (!Type.IsAnArray(array)) throw `LinearToLog: ${array} is not a valid array.`;

    var length = array.length;
    var base_l = 1/Math.log(length); //so the new index without scaling is always between 0 and 1
    var log_array = [];
    var non_empty_indexes = [];

    //re-index
    for (let i = 0; i < length; i++) {
        let log_index = Math.floor( Math.log(i+1)*base_l * length ); //pos * scale
        log_array[log_index] = array[i];
        
        //flag non empty indexes at the same time.
        //it can be done here as log_index grow faster than i.
        if (!Type.IsUndefined(log_array[i])) non_empty_indexes.push(i);
    }

    //interpolate empty indexes
    var j = 0;
    //index 0 is defined and is a starting point for the first interpolation (j=0 to j+1 = 1)
    //If "i" was starting to 0 an unecessary increment would be performed, as the loop would think there was an
    //interpolation done before index 0.
    for (let i = 1; i < length; i++) {
        if (Type.IsUndefined(log_array[i])) {
            //change of area when the right boundary is bypassed.
            //if (i >= non_empty_indexes[j+1]) j++;

            var interpolate = [ log_array[non_empty_indexes[j]], log_array[non_empty_indexes[j+1]] ]; //values to interpolate between.

            var from = interpolate[0];
            var to = interpolate[1];
            var start_index = non_empty_indexes[j];
            var end_index = non_empty_indexes[j+1];
            var current_index = i;
            log_array[i] = from + ((current_index - start_index) / (end_index - start_index)) * (to - from);
            /*
            interpolation between y1 and y2
            x = current index (i)
            x1 = index of y1
            x2 = index of y2
            k = interpolated value
            k = y1 + (x-x1)/(x2-x1) * (y2-y1);
            k = left_value + current_pos/total_length * distance_between_values
            
            example: interpolate from 5 to 10 on array[5,?,?,10]
            x1 = 0 and x2 = 3 (indexes)
            3-0 = 3 steps to reach 10 from 5.
            (array[0] = 5 + 0/3*5 = 5)
            array[1] = 5 + (1-0)/(3-0) * (10-5) = 5 + (1/3 * 5) = 6,6666...
            array[2] = 5 + (2-0)/(3-0) * (10-5) = 5 + (2/3 * 5) = 8.3333...
            (array[3] = 5 + 3/3 * 5 = 10)
            */
        } else {
            //change of area when the right boundary is bypassed.
            j++;
        }
    }

    return log_array;
}

/**
 * returns if the given value is in the interval [min,max] included or excluded;
 *
 * @export
 * @param {Number} value Value to search
 * @param {Array} interval Interval to consider
 * @param {String} type "included" or "excluded"
 * @return {Boolean} 
 */
export function InInterval(value, interval, type) {
    if (!Type.IsANumber(value)) throw `InInterval: ${value} is not a number`;
    if (!Type.IsAnArray(interval)) throw `InInterval: ${interval} is not a valid array`;
    if (!Type.IsUndefined(interval) && interval[0] > interval[1]) throw `InInterval: ${interval} has values in the wrong order. It must be [min,max], min<max`;
    if (Type.IsUndefined(type) || (type !== "included" && type !== "excluded")) throw `InInterval: ${type} is not a valid type. It must be "included" or "excluded"!`;

    switch (type) {
        case "included":
            return (   (value >= interval[0]) && (value <= interval[1])   );
        case "excluded":
            return (   (value > interval[0])  && (value < interval[1])   );
        default:
            throw `InInterval: ${type} is not a valid interval type! (included or excluded)`;
    }
}
