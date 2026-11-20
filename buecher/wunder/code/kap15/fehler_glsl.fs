#version 330
in vec2 fragTexCoord;
out vec4 finalColor;
uniform float zeit;

void main() {
    float hell = 0.5 + 0.5 * sin(zeit)
    finalColor = vec4(hell, hell, hell, 1.0);
}
