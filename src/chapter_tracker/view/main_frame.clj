(ns chapter-tracker.view.main-frame (:gen-class)
  (:use [chapter-tracker file-matching model])
  (:use [chapter-tracker.view
         tools
         create-media-dialog
         create-series-dialog
         series-and-episode-panel
         single-episode-panel
         series-directories-panel])
  (:use clojure.java.io)
)
(import
  '(java.awt GridBagConstraints)
  '(javax.swing JFrame)
)

(defn create-main-frame[]
  (create-frame {:title "Chapter Tracker"}; :width 800 :height 600}
                (let [create-media-button   (action-button "Create Media" (.show (create-create-media-dialog)))
                      [single-episode-panel episode-panel-updating-function]     (create-episode-panel-and-updating-function)
                      [series-directories-panel series-directories-panel-updating-function]     (create-series-directories-panel)

                      [series-and-episode-panel
                       update-serieses-list-function
                       update-episode-table-function
                       get-selected-series-function
                      ] (create-series-and-episode-panel episode-panel-updating-function
                                                         series-directories-panel-updating-function)

                      create-series-button  (action-button "Create Series" (.show (create-create-series-dialog update-serieses-list-function)))

                      rescan-button         (action-button "Rescan" (rescan-all) (update-episode-table-function))

                      delete-series-button  (action-button "Delete Series"
                                                           (when-let [series-to-delete (get-selected-series-function)]
                                                             (.show (create-delete-dialog "series" (.toString series-to-delete)
                                                                                          #(do
                                                                                             (delete-series-record series-to-delete)
                                                                                             (update-serieses-list-function)
                                                                                           )))))
                      download-button       (action-button "Download"
                                                           (when-let [series-to-download (get-selected-series-function)]
                                                             (let [command-dir (:download-command-dir series-to-download)
                                                                   command     (:download-command series-to-download)]
                                                               (when-not (empty? command)
                                                                 (if (empty? command-dir)
                                                                   (future (.exec (Runtime/getRuntime) command))
                                                                   (future (.exec (Runtime/getRuntime)  command nil (file command-dir))))))))
                     ]
                  (.setDefaultCloseOperation frame JFrame/DISPOSE_ON_CLOSE)

                  (add-with-constraints create-media-button
                                        (gridx 0) (gridy 0))

                  (add-with-constraints create-series-button
                                        (gridx 1) (gridy 0))

                  (add-with-constraints delete-series-button
                                        (gridx 2) (gridy 0))

                  (add-with-constraints rescan-button
                                        (gridx 3) (gridy 0))

                  (add-with-constraints download-button
                                        (gridx 4) (gridy 0))

                  (add-with-constraints series-and-episode-panel
                                        (gridx 0) (gridy 1) (gridwidth 5) (gridheight 2) (fill GridBagConstraints/BOTH))

                  (add-with-constraints (create-panel {:width 510 :height 450}
                                                      (add-with-constraints series-directories-panel
                                                                            (gridx 0) (gridy 1) (weighty 0.6) (fill GridBagConstraints/BOTH))

                                                      (add-with-constraints single-episode-panel
                                                                            (gridx 0) (gridy 0) (fill GridBagConstraints/HORIZONTAL))
                                        )
                                        (gridx 5) (gridy 0) (gridheight 3) (fill GridBagConstraints/BOTH))
                )
  )
)
