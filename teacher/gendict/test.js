function left_zero_4(str) {
    if (str != null && str != '' && str != 'undefined') {
      if (str.length == 2) {
        return '00' + str;
      }
    }
    return str;
  }

function unicode(str) {
    var value = '';
    for (var i = 0; i < str.length; i++) {
      value += left_zero_4(parseInt(str.charCodeAt(i)).toString(16));
    }
    return value;
}


console.log(unicode('你'));