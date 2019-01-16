//index.js
//获取应用实例
const app = getApp();

Page({
  data: {},
  onReady: function(){
    wx.redirectTo({
      url: '/teacher/pages/index/index'
    });
  },
  onLoad: function () {
  }
})
