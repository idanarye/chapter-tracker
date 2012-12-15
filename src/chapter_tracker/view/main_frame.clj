(ns chapter-tracker.view.main-frame (:gen-class)
  (:use [chapter-tracker file-matching])
  (:use [chapter-tracker.view tools create-media-dialog create-series-dialog series-and-episode-panel single-episode-panel series-directories-panel])
)
(import
  '(java.awt GridBagConstraints)
  '(javax.swing JFrame)
)

(defn create-main-frame[]
  (create-frame {:title "Chapter Tracker"}; :width 800 :height 600}
                (let [create-media-button   (action-button "Create Media" (.show (create-create-media-dialog)))
                      create-series-button  (action-button "Create Series" (.show (create-create-series-dialog)))
                      rescan-button         (action-button "Rescan" (rescan-all))
                      [single-episode-panel episode-panel-updating-function]     (create-episode-panel-and-updating-function)
                      [series-directories-panel series-directories-panel-updating-function]     (create-series-directories-panel)
                      series-and-episode-panel (create-series-and-episode-panel episode-panel-updating-function series-directories-panel-updating-function)
                     ]
                  (.setDefaultCloseOperation frame JFrame/DISPOSE_ON_CLOSE)

                  (add-with-constraints create-media-button
                                        (gridx 0) (gridy 0))

                  (add-with-constraints create-series-button
                                        (gridx 1) (gridy 0))

                  (add-with-constraints rescan-button
                                        (gridx 2) (gridy 0))

                  (add-with-constraints series-and-episode-panel
                                        (gridx 0) (gridy 1) (gridwidth 3) (gridheight 2) (fill GridBagConstraints/BOTH))

                  (add-with-constraints (create-panel {:width 500 :height 450}
                                                      (add-with-constraints series-directories-panel
                                                                            (gridx 0) (gridy 1) (weighty 0.6) (fill GridBagConstraints/BOTH))

                                                      (add-with-constraints single-episode-panel
                                                                            (gridx 0) (gridy 0) (fill GridBagConstraints/HORIZONTAL))
                                        )
                                        (gridx 3) (gridy 0) (gridheight 3) (fill GridBagConstraints/BOTH))
                )
  )
)
