import * as React from "react"
import { useEffect } from "react"

// https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Classes
class vec3
{
    constructor(x, y, z) {
        this.vec = [x, y, z];
    }

    get x()
    {
        return this.vec[0];
    }

    get y()
    {
        return this.vec[1];
    }

    get z()
    {
        return this.vec[2];
    }
}

class mat4
{
    constructor() {
        this.mat = [
            /* 0 */ 0.0, 0.0, 0.0, 0.0, // 3
            /* 4 */ 0.0, 0.0, 0.0, 0.0, // 7
            /* 8 */ 0.0, 0.0, 0.0, 0.0, // 11
            /* 12*/ 0.0, 0.0, 0.0, 0.0, // 15
        ];
    }

    static identity()
    {
        const result = new mat4();
        result.mat[0] = 1.0;
        result.mat[5] = 1.0;
        result.mat[10] = 1.0;
        result.mat[15] = 1.0;
        return result;
    }

    // Creates a translation matrix
    static translate(vec)
    {
        const result = mat4.identity();

        result.mat[12] = vec.x;
        result.mat[13] = vec.y;
        result.mat[14] = vec.z;
        result.mat[15] = 1.0;

        return result;
    }

    // Creates a scaling matrix
    static scale(x, y, z)
    {
        const result = mat4.identity();

        result.mat[0] = x;
        result.mat[5] = y;
        result.mat[10] = z;
        result.mat[15] = 1.0;

        return result;
    }

    // Creates a rotation matrix
    static rotate(x, y, z)
    {
        const rotX = mat4.identity();
        const rotY = mat4.identity();
        const rotZ = mat4.identity();

        const radPerDegree = 2*Math.PI / 360.0;

        const radX = x * radPerDegree;
        const radY = y * radPerDegree;
        const radZ = z * radPerDegree;

        rotX.mat[5] = Math.cos(radX);
        rotX.mat[6] = Math.sin(radX);
        rotX.mat[9] = -Math.sin(radX);
        rotX.mat[10] = Math.cos(radX);

        rotY.mat[0] = Math.cos(radY);
        rotY.mat[2] = -Math.sin(radY);
        rotY.mat[8] = Math.sin(radY);
        rotY.mat[10] = Math.cos(radY);

        rotZ.mat[0] = Math.cos(radZ);
        rotZ.mat[1] = Math.sin(radZ);
        rotZ.mat[4] = -Math.sin(radZ);
        rotZ.mat[5] = Math.cos(radZ);

        const result = new mat4();
        result.mat = matmul(matmul(rotX, rotY), rotZ);
        return result;
    }

    static perspective(fov, aspect, near = 1.0, far = 45.0)
    {
        const radPerDegree = Math.PI / 180.0;
        const fFovRad = fov * radPerDegree;
        const frustumScale = 1.0 / Math.tan(fFovRad / 2.0);

        const result = new mat4();

        result.mat[0] = frustumScale / aspect;
        result.mat[5] = frustumScale;
        result.mat[10] = (far + near) / (near - far);
        result.mat[11] = -1.0;
        result.mat[14] = (2*far*near) / (near - far);

        return result;
    }
}

function matmul(left, right)
{
    // use SIMD in the future
    const result = new mat4();

    // first row
    result.mat[0] =
        left.mat[0]*right.mat[0] +
        left.mat[1]*right.mat[4] +
        left.mat[2]*right.mat[8] +
        left.mat[3]*right.mat[12];

    result.mat[1] =
        left.mat[0]*right.mat[1] +
        left.mat[1]*right.mat[5] +
        left.mat[2]*right.mat[9] +
        left.mat[3]*right.mat[13];

    result.mat[2] =
        left.mat[0]*right.mat[2] +
        left.mat[1]*right.mat[6] +
        left.mat[2]*right.mat[10] +
        left.mat[3]*right.mat[14];

    result.mat[3] =
        left.mat[0]*right.mat[3] +
        left.mat[1]*right.mat[7] +
        left.mat[2]*right.mat[11] +
        left.mat[3]*right.mat[15];


    // second row

    result.mat[4] =
        left.mat[4]*right.mat[0] +
        left.mat[5]*right.mat[4] +
        left.mat[6]*right.mat[8] +
        left.mat[7]*right.mat[12];

    result.mat[5] =
        left.mat[4]*right.mat[1] +
        left.mat[5]*right.mat[5] +
        left.mat[6]*right.mat[9] +
        left.mat[7]*right.mat[13];

    result.mat[6] =
        left.mat[4]*right.mat[2] +
        left.mat[5]*right.mat[6] +
        left.mat[6]*right.mat[10] +
        left.mat[7]*right.mat[14];

    result.mat[7] =
        left.mat[4]*right.mat[3] +
        left.mat[5]*right.mat[7] +
        left.mat[6]*right.mat[11] +
        left.mat[7]*right.mat[15];

    // third row
    result.mat[8] =
        left.mat[8]*right.mat[0] +
        left.mat[9]*right.mat[4] +
        left.mat[10]*right.mat[8] +
        left.mat[11]*right.mat[12];

    result.mat[5] =
        left.mat[8]*right.mat[1] +
        left.mat[9]*right.mat[5] +
        left.mat[10]*right.mat[9] +
        left.mat[11]*right.mat[13];

    result.mat[6] =
        left.mat[8]*right.mat[2] +
        left.mat[9]*right.mat[6] +
        left.mat[10]*right.mat[10] +
        left.mat[11]*right.mat[14];

    result.mat[7] =
        left.mat[8]*right.mat[3] +
        left.mat[9]*right.mat[7] +
        left.mat[10]*right.mat[11] +
        left.mat[11]*right.mat[15];

    // fourth row
    result.mat[12] =
        left.mat[12]*right.mat[0] +
        left.mat[13]*right.mat[4] +
        left.mat[14]*right.mat[8] +
        left.mat[15]*right.mat[12];

    result.mat[13] =
        left.mat[12]*right.mat[1] +
        left.mat[13]*right.mat[5] +
        left.mat[14]*right.mat[9] +
        left.mat[15]*right.mat[13];

    result.mat[14] =
        left.mat[12]*right.mat[2] +
        left.mat[13]*right.mat[6] +
        left.mat[14]*right.mat[10] +
        left.mat[15]*right.mat[14];

    result.mat[15] =
        left.mat[12]*right.mat[3] +
        left.mat[13]*right.mat[7] +
        left.mat[14]*right.mat[11] +
        left.mat[15]*right.mat[15];

    return result;
}

const canvasStyle = {
    width: 640,
    height: 480
}

function getRenderingContext()
{
    var canvas = document.querySelector("canvas");
    canvas.width = canvas.clientWidth;
    canvas.height = canvas.clientHeight;
    var gl = canvas.getContext("webgl");
    if (!gl)
    {
        var paragraph = document.querySelector("p");
        paragraph.innerText = "Failed to get WebGL context";
        return null;
    }

    gl.viewport(0, 0, gl.drawingBufferWidth, gl.drawingBufferHeight);
    gl.clearColor(0.0, 0.0, 0.0, 1.0);
    gl.clear(gl.COLOR_BUFFER_BIT);
    return gl;
}

var gl;
var timer;
var buffers;
var programInfo;

const vertexShader = `
attribute vec4 aVertexPosition;
attribute vec4 aVertexColor;
uniform mat4 uModelViewMatrix;
uniform mat4 uProjectionMatrix;
varying lowp vec4 vColor;
void main(void) {
  gl_Position = uProjectionMatrix * uModelViewMatrix * aVertexPosition;
  vColor = aVertexColor;
}
`;

const fragmentShader = `
varying lowp vec4 vColor;
void main(void) {
  gl_FragColor = vColor;
}
`;


function init()
{
    if (!(gl = getRenderingContext()))
    {
        return;
    }

    programInfo = initShaders();

    buffers = initBuffers();

    draw();

    //cleanup();
}

function initShaders()
{
    var vs = gl.createShader(gl.VERTEX_SHADER);
    gl.shaderSource(vs, vertexShader);
    gl.compileShader(vs);
    if (!gl.getShaderParameter(vs, gl.COMPILE_STATUS))
    {
        console.error(gl.getShaderInfoLog(vs));
        alert("error compiling shader");
        gl.deleteShader(vs);
        return;
    }

    var fs = gl.createShader(gl.FRAGMENT_SHADER);
    gl.shaderSource(fs, fragmentShader);
    gl.compileShader(fs);
    if (!gl.getShaderParameter(fs, gl.COMPILE_STATUS))
    {
        console.error(gl.getShaderInfoLog(fs));
        alert("error compiling shader");
        gl.deleteShader(fs);
        return;
    }


    var program = gl.createProgram();
    gl.attachShader(program, vs);
    gl.attachShader(program, fs);
    gl.linkProgram(program);

    gl.detachShader(program, vs);
    gl.detachShader(program, fs);
    gl.deleteShader(vs);
    gl.deleteShader(fs);

    if (!gl.getProgramParameter(program, gl.LINK_STATUS)) {
        var linkErrorLog = gl.getProgramInfoLog(program);
        console.log(linkErrorLog);
        cleanup();
        return;
    }

    return {
        program: program,
        attribLocations: {
            vertexPosition: gl.getAttribLocation(program, 'aVertexPosition'),
            vertexColor: gl.getAttribLocation(program, 'aVertexColor'),
        },
        uniformLocations: {
            projectionMatrix: gl.getUniformLocation(program, 'uProjectionMatrix'),
            modelViewMatrix: gl.getUniformLocation(program, 'uModelViewMatrix'),
        }
    };
}

function initBuffers()
{
    const positions = [
        1.0,  1.0,
       -1.0,  1.0,
        1.0, -1.0,
       -1.0, -1.0,
     ];
    const positionBuffer = gl.createBuffer();
    gl.bindBuffer(gl.ARRAY_BUFFER, positionBuffer);
    gl.bufferData(gl.ARRAY_BUFFER, new Float32Array(positions), gl.STATIC_DRAW);

     var colors = [
        1.0,  1.0,  1.0,  1.0,    // white
        1.0,  0.0,  0.0,  1.0,    // red
        0.0,  1.0,  0.0,  1.0,    // green
        0.0,  0.0,  1.0,  1.0,    // blue
      ];
    const colorBuffer = gl.createBuffer();
    gl.bindBuffer(gl.ARRAY_BUFFER, colorBuffer);
    gl.bufferData(gl.ARRAY_BUFFER, new Float32Array(colors), gl.STATIC_DRAW);
    
    return {
        position: positionBuffer,
        color: colorBuffer
    }
}

function draw()
{
    gl.clearColor(0.0, 0.0, 0.0, 1.0);
    gl.clearDepth(1.0);
    gl.enable(gl.DEPTH_TEST);
    gl.depthFunc(gl.LEQUAL);

    gl.clear(gl.COLOR_BUFFER_BIT | gl.DEPTH_BUFFER_BIT);

    const fieldOfView = 45.0;
    const aspect = gl.canvas.clientWidth / gl.canvas.clientHeight;
    const near = 0.1;
    const far = 100.0;
    const projection = mat4.perspective(fieldOfView, aspect, near , far);

    const modelView = mat4.translate(new vec3(0.0, 0.0, -6.0));

    {
        const numComponents = 2;
        const type = gl.FLOAT;
        const normalized = false;
        const stride = 0;
        const offset = 0;
        gl.bindBuffer(gl.ARRAY_BUFFER, buffers.position);
        gl.vertexAttribPointer(
            programInfo.attribLocations.vertexPosition,
            numComponents,
            type,
            normalized,
            stride,
            offset
        );
        gl.enableVertexAttribArray(programInfo.attribLocations.vertexPosition);
    }

    {
        const numComponents = 4;
        const type = gl.FLOAT;
        const normalized = false;
        const stride = 0;
        const offset = 0;
        gl.bindBuffer(gl.ARRAY_BUFFER, buffers.color);
        gl.vertexAttribPointer(
            programInfo.attribLocations.vertexColor,
            numComponents,
            type,
            normalized,
            stride,
            offset
        );
        gl.enableVertexAttribArray(programInfo.attribLocations.vertexColor);
    }

    gl.useProgram(programInfo.program);
    gl.uniformMatrix4fv(
        programInfo.uniformLocations.projectionMatrix,
        false,
        projection.mat
    );
    gl.uniformMatrix4fv(
        programInfo.uniformLocations.modelViewMatrix,
        false,
        modelView.mat
    );

    {
        const offset = 0;
        const vertexCount = 4;
        gl.drawArrays(gl.TRIANGLE_STRIP, offset, vertexCount);
    }

    //timer = setTimeout(draw, 17);
    cleanup();
}

function cleanup()
{
    gl.useProgram(null);
    if (buffers.position) gl.deleteBuffer(buffers.position);
    if (buffers.color) gl.deleteBuffer(buffers.color);
    if (programInfo.program) gl.deleteProgram(programInfo.program);
}

const ArcadePage = () => {

    useEffect(() => {
        init()

        return cleanup;
    })

    return (
        <main>

            <title>Arcade</title>
            <h1>Arcade</h1>
            <p>Arcade</p>
            <canvas style={canvasStyle} id="canvas">Your browser does not support HTML5 canvas.</canvas>
        </main>
    )
}

export default ArcadePage