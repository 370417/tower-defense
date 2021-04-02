precision mediump float;

varying vec2 vTextureCoord;
varying vec4 vColor;

uniform sampler2D uSampler;
uniform float customUniform;

void main(void)
{
    vec2 uvs = vTextureCoord.xy;

    vec4 fg = texture2D(uSampler, vTextureCoord);

    float val = fg.r > customUniform ? 1.0 : 0.0; // * fg.a

    fg.r = val;
    fg.g = val;
    fg.b = val;

    //fg.r = clamp(fg.r,0.0,0.9);

    gl_FragColor = fg;

}
