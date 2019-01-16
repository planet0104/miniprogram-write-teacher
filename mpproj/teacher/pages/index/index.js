// teacher/pages/index.js
var title = "呵呵";
require("teacher.js");
var util = require("util.js");

Page({

  /**
   * 页面的初始数据
   */
  data: {
    showShare: false,
    showInput: false,
    inputContent: "",

    showModal: false,
    modalText: "提示",
    modalConfirm: "确定",

    //完成进度预览
    words: [],
    showCanvasPlace: false,
    showMenu: false,
    showChoose: false,
    exerciseValue: [],
    exerciseGroups: [],
    exerciseNames: [],

    brushDotHidden: true,
    brushDotBackground: 'rgba(255,255,0,.4)',

    rewriteTipHidden: true,
    rewriteTipAnimation: "",

    histories: [],
    showHistoryStroke: false,
    historyStrokeText: [],
    historyStrokeImageSrc: "",

    hideScoreToast: true,
    scoreToastImage: "/static/star_2.png",
    scoreToastText: "",
    scoreToastAnimation: "showtoast 1000ms",
    historyStrokeScoreImage: "",
    historyStrokeScoreText: "",
  },

  inputFeedback: function(){
    this.hideMenu();
    getApp().add_feedback();
  },

  addFeedback: function(feedback, callback){
    const db = wx.cloud.database();
    db.collection('feedbacks').add({
      data: {
        description: feedback,
        due: new Date()
      },
      success(res) {
        callback(true);
      },
      fail: function(res){
        callback(false);
        console.log(res);
      }
    });
  },

  showMenu: function(){
    this.setData({ showMenu: true });
  },
  hideMenu: function(){
    this.setData({ showMenu: false });
  },

  showShare: function(){
    this.setData({ showCanvasPlace: true, showMenu: false, showShare: true });
  },

  //显示选择对话框
  showChoose: function(){
    getApp().show_choose();
  },
  //关闭选择对话框
  hideChoose: function(){
    this.setData({ showChoose: false, showCanvasPlace: false, });
  },
  hideChooseAndSet: function () {
    this.setData({ showChoose: false, showCanvasPlace: false, });
    getApp().handle_choose(this.data.exerciseValue[0], this.data.exerciseValue[1]);
  },
  bindChooseChange: function(e){
    const val = e.detail.value;
    getApp().handle_choose_change(this.data.exerciseValue[0], this.data.exerciseValue[1], val[0], val[1]);
  },
  //搜索选择对话框
  searchTitle: function(e){
    getApp().search(e.detail.value);
  },

  //显示练习记录
  showHistory: function(){
    getApp().histories();
  },
  drawStrokes: function (history, index){
    //画布大小计算
    var canvasWidth = (wx.getSystemInfoSync().windowWidth / 750) * 400;//400rpx
    var scale = canvasWidth/320.0;
    var canvasContext = wx.createCanvasContext("strokeCanvas");
    //------------------ 绘制用户的笔画 -----------------------------
    canvasContext.setStrokeStyle("#333");
    canvasContext.setFillStyle("#333");
    canvasContext.scale(scale, scale);
    var lineWidth = 15;
    canvasContext.setLineWidth(lineWidth);
    //选择对应得笔画
    let strokes = history.strokes[index];
    let score = history.scores[index];
    for(var i=0; i<strokes.length; i++){
      var points = strokes[i];
      let slen = points.length;
      //--------------画圆--------------
      canvasContext.save();
      canvasContext.beginPath();
      canvasContext.arc(points[0].x, points[0].y, lineWidth/2, 0.0, 360.0, false);
      canvasContext.fill();
      canvasContext.restore();
      //--------------------------------

      canvasContext.beginPath();
      canvasContext.moveTo(points[0].x, points[0].y);
      for (var p=0; p<points.length; p++) {
        canvasContext.lineTo(points[p].x, points[p].y);
      }
      canvasContext.stroke();
      //--------------画圆--------------
      canvasContext.save();
      canvasContext.beginPath();
      canvasContext.arc(points[slen - 1].x, points[slen - 1].y, lineWidth / 2, 0.0, 360.0, false);
      canvasContext.fill();
      canvasContext.restore();
      //--------------------------------
    }
    canvasContext.draw();
    //显示当前字的得分
    var image = "";
    if (score >= 8.5){
      image = "/static/star_3.png";
    } else if (score >= 7.5){
      image = "/static/star_2.png";
    } else {
      image = "/static/star_1.png";
    };
    this.setData({
      historyStrokeScoreImage: image,
      historyStrokeScoreText: score.toFixed(1)+"分",
    });
  },
  //隐藏练习记录笔画预览对话框
  hideHistoryStroke: function(){
    this.setData({ showCanvasPlace: false,showHistoryStroke: false });
  },
  //历史记录笔画选择以后，生成图片
  onHistoryStrokesChange: function(e){
    getApp().handle_history(e.detail.value);
  },

  previousCharacter: function(){
    this.hideMenu();
    getApp().previous();
  },

  about: function(){
    this.setData({
      showModal: true,
      modalConfirm: "确定",
      modalText: "\r\n识字画板\r\n\r\n作者:planet2@qq.com\r\n\r\n",
    });
  },

  rewrite: function(){
    getApp().eraser();
  },

  queryDict: function(){
    getApp().query_dict();
  },

  //开始/停止 动画演示
  strokeAnim: function(){
    getApp().stroke_anim();
  },

  showModal : function (obj) {
    getApp().page.modalObject = obj;
    getApp().page.setData({
      showModal: true,
      modalConfirm: obj.confirmText,
      modalText: obj.content,
    });
  },

  onReady: function(){
    //console.log("onReady>>>", new Date(), Date.now());
    getApp().page = this;
    getApp().canvasContext = wx.createCanvasContext("firstCanvas");

    let rpx = 620;
    let width = rpx / 750 * wx.getSystemInfoSync().windowWidth;

    getApp().page.setData({ renderSize: width });
    //console.log("call on_ready>>>", new Date(), Date.now());
    getApp().on_ready();

    /*
    wx.getStorage({
      key: 'showtip',
      fail: function(res) {
        wx.setStorage({
          key: 'showtip',
          data: 'showtip',
        });
        getApp().page.showModal({
          showCancel: false,
          confirmText: "确定",
          title: "提示",
          content: "使用电容笔练字更方便！",
          success: function (res) {
          }
        });
      }
    });
    */
  },

  /**
   * 生命周期函数--监听页面加载
   */
  onLoad: function (options) {
  },

  /**
   * 用户点击右上角分享
   */
  onShareAppMessage: function () {
    this.hideShare();
    return {
      title: '快来使用识字画板',
      path: 'pages/index/index',
    }
  },

  hideShare: function(){
    this.setData({ showCanvasPlace: false, showShare: false });
  },

  /**
   * 弹出框蒙层截断touchmove事件
   */
  preventTouchMove: function () {},

  hideInput: function(){
    this.setData({ showInput: false, showCanvasPlace: false, });
  },
  showInput: function(){
    var page = this;
    this.hideMenu();
    getApp().show_input();
  },
  onInputChange: function(e){
    //缓存数据
    this.setData({ inputContent: e.detail.value});
  },
  confirmInput: function(){
    this.hideInput();
    var text = this.data.inputContent;
    getApp().handle_input(text);
  },

  /**
   * 隐藏模态对话框
   */
  hideModal: function () {
    if (this.modalObject) this.modalObject.success();
    this.setData({ showModal: false });
  },
  /**
   * 对话框确认按钮点击事件
   */
  onConfirm: function () {
    this.hideModal();
  },
  onTouchstart: function(e){
    var touch = e.touches[0];
    getApp().on_touchstart(touch.x, touch.y, e.target.offsetLeft, e.target.offsetTop);
  },
  onTouchmove: function (e) {
    var touch = e.touches[0];
    getApp().on_touchmove(touch.x, touch.y, e.target.offsetLeft, e.target.offsetTop);
  },
  onTouchcancel: function () { getApp().on_touchcancel(); },
  onTouchend: function () { getApp().on_touchend(); },
})