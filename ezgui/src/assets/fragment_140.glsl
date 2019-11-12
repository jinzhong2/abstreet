#version 140

// (x offset, y offset, zoom)
uniform vec3 transform;
// (window width, window height, _)
uniform vec3 window;

uniform sampler2D tex0;
uniform sampler2D tex1;
uniform sampler2D tex2;
uniform sampler2D tex3;
uniform sampler2D tex4;
uniform sampler2D tex5;
uniform sampler2D tex6;
uniform sampler2D tex7;
uniform sampler2D tex8;
uniform sampler2D tex9;
uniform sampler2D tex10;
uniform sampler2D tex11;
uniform sampler2D tex12;
uniform sampler2D tex13;
uniform sampler2D tex14;
uniform sampler2D tex15;
uniform sampler2D tex16;
uniform sampler2D tex17;
uniform sampler2D tex18;
uniform sampler2D tex19;

in vec4 pass_style;
out vec4 f_color;

void main() {
    if (pass_style[3] != 0.0) {
        f_color = pass_style;
    } else if (pass_style[0] == 0.0) {
        f_color = texture(tex0, vec2(pass_style[1], pass_style[2]));
    } else if (pass_style[0] == 1.0) {
        f_color = texture(tex1, vec2(pass_style[1], pass_style[2]));
    } else if (pass_style[0] == 2.0) {
        f_color = texture(tex2, vec2(pass_style[1], pass_style[2]));
    } else if (pass_style[0] == 3.0) {
        f_color = texture(tex3, vec2(pass_style[1], pass_style[2]));
    } else if (pass_style[0] == 4.0) {
        f_color = texture(tex4, vec2(pass_style[1], pass_style[2]));
    } else if (pass_style[0] == 5.0) {
        f_color = texture(tex5, vec2(pass_style[1], pass_style[2]));
    } else if (pass_style[0] == 6.0) {
        f_color = texture(tex6, vec2(pass_style[1], pass_style[2]));
    } else if (pass_style[0] == 7.0) {
        f_color = texture(tex7, vec2(pass_style[1], pass_style[2]));
    } else if (pass_style[0] == 8.0) {
        f_color = texture(tex8, vec2(pass_style[1], pass_style[2]));
    } else if (pass_style[0] == 9.0) {
        f_color = texture(tex9, vec2(pass_style[1], pass_style[2]));
    } else if (pass_style[0] == 10.0) {
        f_color = texture(tex10, vec2(pass_style[1], pass_style[2]));
    } else if (pass_style[0] == 11.0) {
        f_color = texture(tex11, vec2(pass_style[1], pass_style[2]));
    } else if (pass_style[0] == 12.0) {
        f_color = texture(tex12, vec2(pass_style[1], pass_style[2]));
    } else if (pass_style[0] == 13.0) {
        f_color = texture(tex13, vec2(pass_style[1], pass_style[2]));
    } else if (pass_style[0] == 14.0) {
        f_color = texture(tex14, vec2(pass_style[1], pass_style[2]));
    } else if (pass_style[0] == 15.0) {
        f_color = texture(tex15, vec2(pass_style[1], pass_style[2]));
    } else if (pass_style[0] == 16.0) {
        f_color = texture(tex16, vec2(pass_style[1], pass_style[2]));
    } else if (pass_style[0] == 17.0) {
        f_color = texture(tex17, vec2(pass_style[1], pass_style[2]));
    } else if (pass_style[0] == 18.0) {
        f_color = texture(tex18, vec2(pass_style[1], pass_style[2]));
    } else if (pass_style[0] == 19.0) {
        f_color = texture(tex19, vec2(pass_style[1], pass_style[2]));
    } else if (pass_style[0] == 100.0) {
        // The hatching should be done in map-space, so panning/zooming doesn't move the stripes.
        // This is screen_to_map, also accounting for the y-inversion done by the vertex shader.
        float map_x = (gl_FragCoord.x + transform[0]) / transform[2];
        float map_y = (window[1] - gl_FragCoord.y + transform[1]) / transform[2];
        if (mod(map_x + map_y, 2.0) <= 0.1) {
            f_color = vec4(0.0, 1.0, 1.0, 1.0);
        } else if (mod(map_x - map_y, 2.0) <= 0.1) {
            f_color = vec4(0.0, 1.0, 1.0, 1.0);
        } else {
            // Let the polygon with its original colors show instead.
            discard;
	}
    } else if (pass_style[0] == 101.0) {
        float map_x = (gl_FragCoord.x + transform[0]) / transform[2];
        float map_y = (window[1] - gl_FragCoord.y + transform[1]) / transform[2];
        if (mod(map_x + map_y, 2.0) <= 0.5) {
            f_color = vec4(1.0, 1.0, 1.0, 1.0);
        } else {
            // Let the polygon with its original colors show instead.
            discard;
	}
    }
}
